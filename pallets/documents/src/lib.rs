//! # Pallet Documents
//!
//! Manages document hash registration with version tracking, supersession
//! chains, and filing readiness workflow. Each document is linked to a matter.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditHook, ContentHash, DocumentId, DocumentStatus, FilingReadiness, MatterId,
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

        /// Maximum length of a document title (bytes).
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct DocumentRecord<T: Config> {
        pub id: DocumentId,
        pub matter_id: MatterId,
        pub title_hash: ContentHash,
        pub content_hash: ContentHash,
        pub encrypted_uri: BoundedVec<u8, T::MaxUriLength>,
        pub document_type_hash: ContentHash,
        pub version: u32,
        pub status: DocumentStatus,
        pub filing_readiness: FilingReadiness,
        pub superseded_by: Option<DocumentId>,
        pub supersedes: Option<DocumentId>,
        pub registered_by: T::AccountId,
        pub registered_at: BlockNumberFor<T>,
        pub updated_at: BlockNumberFor<T>,
        pub metadata_hash: ContentHash,
    }

    // ─── Storage ───────────────────────────────────────────────────

    #[pallet::storage]
    pub type NextDocumentId<T> = StorageValue<_, DocumentId, ValueQuery>;

    #[pallet::storage]
    pub type Documents<T: Config> =
        StorageMap<_, Blake2_128Concat, DocumentId, DocumentRecord<T>, OptionQuery>;

    #[pallet::storage]
    pub type DocumentsByMatter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        DocumentId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new document version was registered.
        DocumentRegistered {
            document_id: DocumentId,
            matter_id: MatterId,
            registrar: T::AccountId,
            content_hash: ContentHash,
            version: u32,
        },
        /// A document was superseded by a newer version.
        DocumentSuperseded {
            document_id: DocumentId,
            superseded_by: DocumentId,
            actor: T::AccountId,
        },
        /// Document filing readiness status changed.
        FilingReadinessChanged {
            document_id: DocumentId,
            actor: T::AccountId,
            new_readiness: FilingReadiness,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The specified document does not exist.
        DocumentNotFound,
        /// Caller is not authorized for this operation.
        NotAuthorized,
        /// The document has already been superseded.
        AlreadySuperseded,
        /// Cannot supersede a document from a different matter.
        MatterMismatch,
        /// Invalid version number.
        InvalidVersion,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new document (version 1) linked to a matter.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn register_document(
            origin: OriginFor<T>,
            matter_id: MatterId,
            title_hash: ContentHash,
            content_hash: ContentHash,
            encrypted_uri: BoundedVec<u8, T::MaxUriLength>,
            document_type_hash: ContentHash,
            metadata_hash: ContentHash,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();

            let document_id = NextDocumentId::<T>::get();
            let next_id = document_id
                .checked_add(1)
                .ok_or(sp_runtime::ArithmeticError::Overflow)?;

            let record = DocumentRecord::<T> {
                id: document_id,
                matter_id,
                title_hash,
                content_hash,
                encrypted_uri,
                document_type_hash,
                version: 1,
                status: DocumentStatus::Draft,
                filing_readiness: FilingReadiness::NotReady,
                superseded_by: None,
                supersedes: None,
                registered_by: registrar.clone(),
                registered_at: now,
                updated_at: now,
                metadata_hash,
            };

            Documents::<T>::insert(document_id, &record);
            DocumentsByMatter::<T>::insert(matter_id, document_id, ());
            NextDocumentId::<T>::put(next_id);

            Self::deposit_event(Event::DocumentRegistered {
                document_id,
                matter_id,
                registrar: registrar.clone(),
                content_hash,
                version: 1,
            });

            T::AuditHook::on_state_change(
                Some(matter_id),
                &registrar,
                ActionType::Register,
                SubjectType::Document,
                document_id,
                None,
                Some(content_hash),
            );

            Ok(())
        }

        /// Supersede an existing document with a new version.
        ///
        /// Creates a new document record and links it to the old one.
        /// Both must belong to the same matter.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn supersede_document(
            origin: OriginFor<T>,
            old_document_id: DocumentId,
            new_content_hash: ContentHash,
            new_encrypted_uri: BoundedVec<u8, T::MaxUriLength>,
            new_metadata_hash: ContentHash,
        ) -> DispatchResult {
            let actor = ensure_signed(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();

            // Read the old document
            let old_record =
                Documents::<T>::get(old_document_id).ok_or(Error::<T>::DocumentNotFound)?;
            ensure!(old_record.registered_by == actor, Error::<T>::NotAuthorized);
            ensure!(
                old_record.superseded_by.is_none(),
                Error::<T>::AlreadySuperseded
            );

            // Create the new document
            let new_document_id = NextDocumentId::<T>::get();
            let next_id = new_document_id
                .checked_add(1)
                .ok_or(sp_runtime::ArithmeticError::Overflow)?;

            let new_version = old_record
                .version
                .checked_add(1)
                .ok_or(Error::<T>::InvalidVersion)?;

            let new_record = DocumentRecord::<T> {
                id: new_document_id,
                matter_id: old_record.matter_id,
                title_hash: old_record.title_hash,
                content_hash: new_content_hash,
                encrypted_uri: new_encrypted_uri,
                document_type_hash: old_record.document_type_hash,
                version: new_version,
                status: DocumentStatus::Draft,
                filing_readiness: FilingReadiness::NotReady,
                superseded_by: None,
                supersedes: Some(old_document_id),
                registered_by: actor.clone(),
                registered_at: now,
                updated_at: now,
                metadata_hash: new_metadata_hash,
            };

            // Update old document
            Documents::<T>::mutate(old_document_id, |maybe_record| {
                if let Some(record) = maybe_record {
                    record.superseded_by = Some(new_document_id);
                    record.status = DocumentStatus::Superseded;
                    record.updated_at = now;
                }
            });

            // Insert new document
            Documents::<T>::insert(new_document_id, &new_record);
            DocumentsByMatter::<T>::insert(old_record.matter_id, new_document_id, ());
            NextDocumentId::<T>::put(next_id);

            Self::deposit_event(Event::DocumentSuperseded {
                document_id: old_document_id,
                superseded_by: new_document_id,
                actor: actor.clone(),
            });

            Self::deposit_event(Event::DocumentRegistered {
                document_id: new_document_id,
                matter_id: old_record.matter_id,
                registrar: actor.clone(),
                content_hash: new_content_hash,
                version: new_version,
            });

            T::AuditHook::on_state_change(
                Some(old_record.matter_id),
                &actor,
                ActionType::Supersede,
                SubjectType::Document,
                new_document_id,
                Some(old_record.content_hash),
                Some(new_content_hash),
            );

            Ok(())
        }

        /// Update the filing readiness status of a document.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn update_filing_readiness(
            origin: OriginFor<T>,
            document_id: DocumentId,
            new_readiness: FilingReadiness,
        ) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            Documents::<T>::try_mutate(document_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::DocumentNotFound)?;
                ensure!(record.registered_by == actor, Error::<T>::NotAuthorized);

                record.filing_readiness = new_readiness;
                record.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::FilingReadinessChanged {
                    document_id,
                    actor: actor.clone(),
                    new_readiness,
                });

                T::AuditHook::on_state_change(
                    Some(record.matter_id),
                    &actor,
                    ActionType::Update,
                    SubjectType::Document,
                    document_id,
                    None,
                    None,
                );

                Ok(())
            })
        }
    }
}
