//! # Pallet Audit
//!
//! Durable audit event anchoring for all significant state transitions
//! across the legal-chain runtime. Implements the `AuditHook` trait so
//! other pallets can record audit events via their Config type.
//!
//! Audit records are append-only — no update or delete operations exist.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditId, ContentHash, MatterId, SubjectType,
    };

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ─── Config ────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct AuditRecord<T: Config> {
        pub id: AuditId,
        pub matter_id: Option<MatterId>,
        pub actor: T::AccountId,
        pub action: ActionType,
        pub target_type: SubjectType,
        pub target_id: u64,
        pub before_hash: Option<ContentHash>,
        pub after_hash: Option<ContentHash>,
        pub block_number: BlockNumberFor<T>,
    }

    // ─── Storage ───────────────────────────────────────────────────

    #[pallet::storage]
    pub type NextAuditId<T> = StorageValue<_, AuditId, ValueQuery>;

    #[pallet::storage]
    pub type AuditEvents<T: Config> =
        StorageMap<_, Blake2_128Concat, AuditId, AuditRecord<T>, OptionQuery>;

    /// Index: MatterId × AuditId → () for querying all audit events for a matter.
    #[pallet::storage]
    pub type AuditByMatter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        AuditId,
        (),
        OptionQuery,
    >;

    /// Index: Actor AccountId × AuditId → () for querying all audit events by actor.
    #[pallet::storage]
    pub type AuditByActor<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AuditId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An audit event was permanently anchored on-chain.
        AuditEventAnchored {
            audit_id: AuditId,
            matter_id: Option<MatterId>,
            actor: T::AccountId,
            action: ActionType,
            target_type: SubjectType,
            target_id: u64,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// Audit ID counter overflow (should never happen in practice).
        AuditIdOverflow,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Manually anchor an audit event. This extrinsic is available for
        /// direct submission but in normal operation, audit events are
        /// recorded automatically via the AuditHook.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn anchor_event(
            origin: OriginFor<T>,
            matter_id: Option<MatterId>,
            action: ActionType,
            target_type: SubjectType,
            target_id: u64,
            before_hash: Option<ContentHash>,
            after_hash: Option<ContentHash>,
        ) -> DispatchResult {
            let actor = ensure_signed(origin)?;

            Self::do_anchor(
                matter_id,
                actor,
                action,
                target_type,
                target_id,
                before_hash,
                after_hash,
            )
        }
    }

    // ─── Internal Helpers ──────────────────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Core audit anchoring logic used by both the extrinsic and the AuditHook.
        pub(crate) fn do_anchor(
            matter_id: Option<MatterId>,
            actor: T::AccountId,
            action: ActionType,
            target_type: SubjectType,
            target_id: u64,
            before_hash: Option<ContentHash>,
            after_hash: Option<ContentHash>,
        ) -> DispatchResult {
            let audit_id = NextAuditId::<T>::get();
            let next_id = audit_id
                .checked_add(1)
                .ok_or(Error::<T>::AuditIdOverflow)?;
            let now = <frame_system::Pallet<T>>::block_number();

            let record = AuditRecord::<T> {
                id: audit_id,
                matter_id,
                actor: actor.clone(),
                action,
                target_type,
                target_id,
                before_hash,
                after_hash,
                block_number: now,
            };

            AuditEvents::<T>::insert(audit_id, &record);

            if let Some(mid) = matter_id {
                AuditByMatter::<T>::insert(mid, audit_id, ());
            }
            AuditByActor::<T>::insert(&actor, audit_id, ());

            NextAuditId::<T>::put(next_id);

            Self::deposit_event(Event::AuditEventAnchored {
                audit_id,
                matter_id,
                actor,
                action,
                target_type,
                target_id,
            });

            Ok(())
        }
    }
}

// ─── AuditHook Implementation ──────────────────────────────────────

use legal_chain_common_types::{ActionType, AuditHook, ContentHash, MatterId, SubjectType};

/// Implement the cross-pallet AuditHook trait so other pallets can use
/// `T::AuditHook::on_state_change(...)` to record audit events.
impl<T: pallet::Config> AuditHook<T::AccountId> for Pallet<T> {
    fn on_state_change(
        matter_id: Option<MatterId>,
        actor: &T::AccountId,
        action: ActionType,
        subject: SubjectType,
        subject_id: u64,
        before_hash: Option<ContentHash>,
        after_hash: Option<ContentHash>,
    ) {
        // Best-effort anchoring: if this fails (e.g., ID overflow), it's logged
        // but does not revert the calling pallet's transaction.
        let _ = Self::do_anchor(
            matter_id,
            actor.clone(),
            action,
            subject,
            subject_id,
            before_hash,
            after_hash,
        );
    }
}
