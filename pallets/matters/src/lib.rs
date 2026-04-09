//! # Pallet Matters
//!
//! Manages the lifecycle of legal matters: creation, metadata updates,
//! and status transitions. Every mutation emits an event and calls the
//! cross-pallet `AuditHook`.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditHook, ContentHash, MatterId, MatterStatus, MatterType, Sensitivity,
        SubjectType, UpdatedField,
    };


    /// The pallet's storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ─── Config ────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Cross-pallet audit hook. Use `()` for testing.
        type AuditHook: AuditHook<Self::AccountId>;

        /// Maximum length of a matter title (bytes).
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;

        /// Maximum length of a matter description hash or metadata.
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// Maximum number of party account IDs per matter.
        #[pallet::constant]
        type MaxPartiesPerMatter: Get<u32>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    /// On-chain representation of a legal matter.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct MatterRecord<T: Config> {
        pub id: MatterId,
        pub title_hash: ContentHash,
        pub description_hash: ContentHash,
        pub matter_type: MatterType,
        pub status: MatterStatus,
        pub jurisdiction_hash: ContentHash,
        pub sensitivity: Sensitivity,
        pub created_by: T::AccountId,
        pub created_at: BlockNumberFor<T>,
        pub updated_at: BlockNumberFor<T>,
        pub parties: BoundedVec<T::AccountId, T::MaxPartiesPerMatter>,
    }

    // ─── Storage ───────────────────────────────────────────────────

    /// Auto-incrementing matter ID counter.
    #[pallet::storage]
    pub type NextMatterId<T> = StorageValue<_, MatterId, ValueQuery>;

    /// Primary matter storage: MatterId → MatterRecord.
    #[pallet::storage]
    pub type Matters<T: Config> =
        StorageMap<_, Blake2_128Concat, MatterId, MatterRecord<T>, OptionQuery>;

    /// Secondary index: Creator AccountId × MatterId → ().
    #[pallet::storage]
    pub type MattersByCreator<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        MatterId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new matter was created.
        MatterCreated {
            matter_id: MatterId,
            creator: T::AccountId,
            matter_type: MatterType,
            jurisdiction_hash: ContentHash,
        },
        /// Matter metadata was updated.
        MatterUpdated {
            matter_id: MatterId,
            updater: T::AccountId,
            field_changed: UpdatedField,
        },
        /// Matter status changed.
        MatterStatusChanged {
            matter_id: MatterId,
            actor: T::AccountId,
            old_status: MatterStatus,
            new_status: MatterStatus,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The specified matter does not exist.
        MatterNotFound,
        /// Caller is not authorized to modify this matter.
        NotAuthorized,
        /// The requested status transition is not valid.
        InvalidStatusTransition,
        /// The title data exceeds the maximum allowed length.
        TitleTooLong,
        /// The parties list exceeds the maximum allowed count.
        TooManyParties,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new legal matter.
        ///
        /// - `title_hash`: H256 hash of the matter title.
        /// - `description_hash`: H256 hash of the matter description.
        /// - `matter_type`: Category of legal matter.
        /// - `jurisdiction_hash`: H256 hash of the jurisdiction identifier.
        /// - `sensitivity`: Data classification level.
        /// - `parties`: Initial set of party account IDs.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn create_matter(
            origin: OriginFor<T>,
            title_hash: ContentHash,
            description_hash: ContentHash,
            matter_type: MatterType,
            jurisdiction_hash: ContentHash,
            sensitivity: Sensitivity,
            parties: BoundedVec<T::AccountId, T::MaxPartiesPerMatter>,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();

            let matter_id = NextMatterId::<T>::get();
            let next_id = matter_id.checked_add(1).ok_or(sp_runtime::ArithmeticError::Overflow)?;

            let record = MatterRecord::<T> {
                id: matter_id,
                title_hash,
                description_hash,
                matter_type,
                status: MatterStatus::Draft,
                jurisdiction_hash,
                sensitivity,
                created_by: creator.clone(),
                created_at: now,
                updated_at: now,
                parties,
            };

            Matters::<T>::insert(matter_id, &record);
            MattersByCreator::<T>::insert(&creator, matter_id, ());
            NextMatterId::<T>::put(next_id);

            Self::deposit_event(Event::MatterCreated {
                matter_id,
                creator: creator.clone(),
                matter_type,
                jurisdiction_hash,
            });

            T::AuditHook::on_state_change(
                Some(matter_id),
                &creator,
                ActionType::Create,
                SubjectType::Matter,
                matter_id,
                None,
                Some(title_hash),
            );

            Ok(())
        }

        /// Update the title hash of an existing matter.
        ///
        /// Only the matter creator is authorized.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn update_title(
            origin: OriginFor<T>,
            matter_id: MatterId,
            new_title_hash: ContentHash,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Matters::<T>::try_mutate(matter_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::MatterNotFound)?;
                ensure!(record.created_by == caller, Error::<T>::NotAuthorized);

                let old_hash = record.title_hash;
                record.title_hash = new_title_hash;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::MatterUpdated {
                    matter_id,
                    updater: caller.clone(),
                    field_changed: UpdatedField::Title,
                });

                T::AuditHook::on_state_change(
                    Some(matter_id),
                    &caller,
                    ActionType::Update,
                    SubjectType::Matter,
                    matter_id,
                    Some(old_hash),
                    Some(new_title_hash),
                );

                Ok(())
            })
        }

        /// Update the description hash of an existing matter.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn update_description(
            origin: OriginFor<T>,
            matter_id: MatterId,
            new_description_hash: ContentHash,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Matters::<T>::try_mutate(matter_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::MatterNotFound)?;
                ensure!(record.created_by == caller, Error::<T>::NotAuthorized);

                let old_hash = record.description_hash;
                record.description_hash = new_description_hash;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::MatterUpdated {
                    matter_id,
                    updater: caller.clone(),
                    field_changed: UpdatedField::Description,
                });

                T::AuditHook::on_state_change(
                    Some(matter_id),
                    &caller,
                    ActionType::Update,
                    SubjectType::Matter,
                    matter_id,
                    Some(old_hash),
                    Some(new_description_hash),
                );

                Ok(())
            })
        }

        /// Change the status of a matter. Enforces valid state transitions.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn change_status(
            origin: OriginFor<T>,
            matter_id: MatterId,
            new_status: MatterStatus,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Matters::<T>::try_mutate(matter_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::MatterNotFound)?;
                ensure!(record.created_by == caller, Error::<T>::NotAuthorized);
                ensure!(
                    record.status.can_transition_to(&new_status),
                    Error::<T>::InvalidStatusTransition
                );

                let old_status = record.status;
                record.status = new_status;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::MatterStatusChanged {
                    matter_id,
                    actor: caller.clone(),
                    old_status,
                    new_status,
                });

                T::AuditHook::on_state_change(
                    Some(matter_id),
                    &caller,
                    ActionType::StatusChange,
                    SubjectType::Matter,
                    matter_id,
                    None,
                    None,
                );

                Ok(())
            })
        }

        /// Update the sensitivity classification of a matter.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn update_sensitivity(
            origin: OriginFor<T>,
            matter_id: MatterId,
            new_sensitivity: Sensitivity,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Matters::<T>::try_mutate(matter_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::MatterNotFound)?;
                ensure!(record.created_by == caller, Error::<T>::NotAuthorized);

                record.sensitivity = new_sensitivity;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::MatterUpdated {
                    matter_id,
                    updater: caller.clone(),
                    field_changed: UpdatedField::Sensitivity,
                });

                T::AuditHook::on_state_change(
                    Some(matter_id),
                    &caller,
                    ActionType::Update,
                    SubjectType::Matter,
                    matter_id,
                    None,
                    None,
                );

                Ok(())
            })
        }
    }
}
