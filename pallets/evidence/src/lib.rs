//! # Pallet Evidence
//!
//! Manages evidence hash registration, independent verification, and
//! chain-of-custody state tracking. All evidence is linked to a matter.
//! No raw content is stored on-chain — only H256 content hashes and
//! encrypted storage URIs.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditHook, ContentHash, CustodyState, EvidenceId, EvidenceStatus, MatterId,
        SubjectType,
    };

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ─── Config ────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AuditHook: AuditHook<Self::AccountId>;

        /// Maximum length of an encrypted storage URI (bytes).
        #[pallet::constant]
        type MaxUriLength: Get<u32>;

        /// Maximum length of metadata (bytes).
        #[pallet::constant]
        type MaxMetadataLength: Get<u32>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct EvidenceRecord<T: Config> {
        pub id: EvidenceId,
        pub matter_id: MatterId,
        pub content_hash: ContentHash,
        pub encrypted_uri: BoundedVec<u8, T::MaxUriLength>,
        pub evidence_type_hash: ContentHash,
        pub status: EvidenceStatus,
        pub custody_state: CustodyState,
        pub registered_by: T::AccountId,
        pub registered_at: BlockNumberFor<T>,
        pub updated_at: BlockNumberFor<T>,
        pub verified_by: Option<T::AccountId>,
        pub metadata_hash: ContentHash,
    }

    // ─── Storage ───────────────────────────────────────────────────

    #[pallet::storage]
    pub type NextEvidenceId<T> = StorageValue<_, EvidenceId, ValueQuery>;

    #[pallet::storage]
    pub type EvidenceRecords<T: Config> =
        StorageMap<_, Blake2_128Concat, EvidenceId, EvidenceRecord<T>, OptionQuery>;

    #[pallet::storage]
    pub type EvidenceByMatter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        EvidenceId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Evidence hash was anchored to the chain.
        EvidenceRegistered {
            evidence_id: EvidenceId,
            matter_id: MatterId,
            registrar: T::AccountId,
            content_hash: ContentHash,
        },
        /// Evidence was independently verified.
        EvidenceVerified {
            evidence_id: EvidenceId,
            verifier: T::AccountId,
        },
        /// Evidence custody state changed.
        CustodyStateChanged {
            evidence_id: EvidenceId,
            actor: T::AccountId,
            old_state: CustodyState,
            new_state: CustodyState,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The specified evidence record does not exist.
        EvidenceNotFound,
        /// Caller is not authorized for this operation.
        NotAuthorized,
        /// Evidence has already been verified.
        AlreadyVerified,
        /// The encrypted URI exceeds the maximum allowed length.
        UriTooLong,
        /// The custody state transition is not valid at this time.
        InvalidCustodyTransition,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new piece of evidence linked to a matter.
        ///
        /// The caller becomes the registrar. Only the content hash and
        /// encrypted storage URI are stored on-chain.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn register_evidence(
            origin: OriginFor<T>,
            matter_id: MatterId,
            content_hash: ContentHash,
            encrypted_uri: BoundedVec<u8, T::MaxUriLength>,
            evidence_type_hash: ContentHash,
            metadata_hash: ContentHash,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();

            let evidence_id = NextEvidenceId::<T>::get();
            let next_id = evidence_id
                .checked_add(1)
                .ok_or(sp_runtime::ArithmeticError::Overflow)?;

            let record = EvidenceRecord::<T> {
                id: evidence_id,
                matter_id,
                content_hash,
                encrypted_uri,
                evidence_type_hash,
                status: EvidenceStatus::Submitted,
                custody_state: CustodyState::InPossession,
                registered_by: registrar.clone(),
                registered_at: now,
                updated_at: now,
                verified_by: None,
                metadata_hash,
            };

            EvidenceRecords::<T>::insert(evidence_id, &record);
            EvidenceByMatter::<T>::insert(matter_id, evidence_id, ());
            NextEvidenceId::<T>::put(next_id);

            Self::deposit_event(Event::EvidenceRegistered {
                evidence_id,
                matter_id,
                registrar: registrar.clone(),
                content_hash,
            });

            T::AuditHook::on_state_change(
                Some(matter_id),
                &registrar,
                ActionType::Register,
                SubjectType::Evidence,
                evidence_id,
                None,
                Some(content_hash),
            );

            Ok(())
        }

        /// Mark evidence as independently verified.
        ///
        /// The verifier must be a different account from the registrar.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn verify_evidence(
            origin: OriginFor<T>,
            evidence_id: EvidenceId,
        ) -> DispatchResult {
            let verifier = ensure_signed(origin)?;

            EvidenceRecords::<T>::try_mutate(evidence_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::EvidenceNotFound)?;
                ensure!(record.verified_by.is_none(), Error::<T>::AlreadyVerified);
                ensure!(record.registered_by != verifier, Error::<T>::NotAuthorized);

                record.verified_by = Some(verifier.clone());
                record.status = EvidenceStatus::Verified;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::EvidenceVerified {
                    evidence_id,
                    verifier: verifier.clone(),
                });

                T::AuditHook::on_state_change(
                    Some(record.matter_id),
                    &verifier,
                    ActionType::Verify,
                    SubjectType::Evidence,
                    evidence_id,
                    None,
                    Some(record.content_hash),
                );

                Ok(())
            })
        }

        /// Update the chain-of-custody state for evidence.
        ///
        /// Only the registrar can update custody state.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn update_custody(
            origin: OriginFor<T>,
            evidence_id: EvidenceId,
            new_state: CustodyState,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            EvidenceRecords::<T>::try_mutate(evidence_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::EvidenceNotFound)?;
                ensure!(record.registered_by == caller, Error::<T>::NotAuthorized);

                let old_state = record.custody_state;
                record.custody_state = new_state;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::CustodyStateChanged {
                    evidence_id,
                    actor: caller.clone(),
                    old_state,
                    new_state,
                });

                T::AuditHook::on_state_change(
                    Some(record.matter_id),
                    &caller,
                    ActionType::CustodyTransfer,
                    SubjectType::Evidence,
                    evidence_id,
                    None,
                    None,
                );

                Ok(())
            })
        }
    }
}
