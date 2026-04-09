//! # Pallet Identities
//!
//! Manages legal platform identities: registration, role assignment,
//! jurisdiction scope, organization binding, and revocation.
//! Every mutation emits an event and calls the cross-pallet `AuditHook`.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditHook, ContentHash, CredentialId, IdentityRole, SubjectType,
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

        /// Maximum length of an organization name hash.
        #[pallet::constant]
        type MaxOrgLength: Get<u32>;

        /// Maximum number of jurisdiction scopes per identity.
        #[pallet::constant]
        type MaxJurisdictions: Get<u32>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    /// On-chain legal identity record.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct IdentityRecord<T: Config> {
        pub id: CredentialId,
        pub subject: T::AccountId,
        pub role: IdentityRole,
        pub org_hash: ContentHash,
        pub jurisdiction_hashes: BoundedVec<ContentHash, T::MaxJurisdictions>,
        pub active: bool,
        pub registered_by: T::AccountId,
        pub registered_at: BlockNumberFor<T>,
        pub revoked_at: Option<BlockNumberFor<T>>,
    }

    // ─── Storage ───────────────────────────────────────────────────

    #[pallet::storage]
    pub type NextCredentialId<T> = StorageValue<_, CredentialId, ValueQuery>;

    /// Primary identity store: CredentialId → IdentityRecord.
    #[pallet::storage]
    pub type Identities<T: Config> =
        StorageMap<_, Blake2_128Concat, CredentialId, IdentityRecord<T>, OptionQuery>;

    /// Index: Subject AccountId → list of credential IDs.
    #[pallet::storage]
    pub type IdentitiesBySubject<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        CredentialId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        IdentityRegistered {
            credential_id: CredentialId,
            subject: T::AccountId,
            role: IdentityRole,
            registered_by: T::AccountId,
        },
        IdentityRevoked {
            credential_id: CredentialId,
            revoked_by: T::AccountId,
        },
        RoleUpdated {
            credential_id: CredentialId,
            old_role: IdentityRole,
            new_role: IdentityRole,
            updated_by: T::AccountId,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        IdentityNotFound,
        IdentityAlreadyRevoked,
        NotRegistrar,
        NoJurisdictionsProvided,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new legal identity for an account.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn register(
            origin: OriginFor<T>,
            subject: T::AccountId,
            role: IdentityRole,
            org_hash: ContentHash,
            jurisdiction_hashes: BoundedVec<ContentHash, T::MaxJurisdictions>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!jurisdiction_hashes.is_empty(), Error::<T>::NoJurisdictionsProvided);

            let id = NextCredentialId::<T>::get();
            NextCredentialId::<T>::put(id.wrapping_add(1));
            let now = <frame_system::Pallet<T>>::block_number();

            let record = IdentityRecord {
                id,
                subject: subject.clone(),
                role,
                org_hash,
                jurisdiction_hashes,
                active: true,
                registered_by: who.clone(),
                registered_at: now,
                revoked_at: None,
            };

            Identities::<T>::insert(id, &record);
            IdentitiesBySubject::<T>::insert(&subject, id, ());

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Register,
                SubjectType::Identity,
                id,
                None,
                Some(org_hash),
            );

            Self::deposit_event(Event::IdentityRegistered {
                credential_id: id,
                subject,
                role,
                registered_by: who,
            });

            Ok(())
        }

        /// Revoke an identity credential (registrar only).
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn revoke(
            origin: OriginFor<T>,
            credential_id: CredentialId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut record = Identities::<T>::get(credential_id)
                .ok_or(Error::<T>::IdentityNotFound)?;

            ensure!(record.active, Error::<T>::IdentityAlreadyRevoked);
            ensure!(record.registered_by == who, Error::<T>::NotRegistrar);

            let now = <frame_system::Pallet<T>>::block_number();
            record.active = false;
            record.revoked_at = Some(now);
            Identities::<T>::insert(credential_id, &record);

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Revoke,
                SubjectType::Identity,
                credential_id,
                None,
                None,
            );

            Self::deposit_event(Event::IdentityRevoked {
                credential_id,
                revoked_by: who,
            });

            Ok(())
        }

        /// Update the role of an existing identity (registrar only).
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn update_role(
            origin: OriginFor<T>,
            credential_id: CredentialId,
            new_role: IdentityRole,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut record = Identities::<T>::get(credential_id)
                .ok_or(Error::<T>::IdentityNotFound)?;

            ensure!(record.active, Error::<T>::IdentityAlreadyRevoked);
            ensure!(record.registered_by == who, Error::<T>::NotRegistrar);

            let old_role = record.role;
            record.role = new_role;
            Identities::<T>::insert(credential_id, &record);

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Update,
                SubjectType::Identity,
                credential_id,
                None,
                None,
            );

            Self::deposit_event(Event::RoleUpdated {
                credential_id,
                old_role,
                new_role,
                updated_by: who,
            });

            Ok(())
        }
    }
}
