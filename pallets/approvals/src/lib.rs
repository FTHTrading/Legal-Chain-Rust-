//! # Pallet Approvals
//!
//! Multi-party approval workflows for legal objects. Supports reviewer
//! assignment, individual decisions, quorum requirements, and expiry.
//! Every mutation emits an event and calls the cross-pallet `AuditHook`.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, ApprovalId, ApprovalStatus, AuditHook, ContentHash, MatterId, SubjectType,
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

        /// Maximum number of reviewers per approval request.
        #[pallet::constant]
        type MaxReviewers: Get<u32>;

        /// Number of blocks before an approval request expires (0 = no expiry).
        #[pallet::constant]
        type DefaultExpiryBlocks: Get<BlockNumberFor<Self>>;
    }

    // ─── Storage Types ─────────────────────────────────────────────

    /// On-chain representation of an approval request.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct ApprovalRecord<T: Config> {
        pub id: ApprovalId,
        pub matter_id: MatterId,
        /// What kind of object is being approved (Evidence, Document, etc.).
        pub subject_type: SubjectType,
        pub subject_id: u64,
        pub status: ApprovalStatus,
        /// Hash of the object state at request time (integrity check).
        pub subject_hash: ContentHash,
        pub requester: T::AccountId,
        pub reviewers: BoundedVec<T::AccountId, T::MaxReviewers>,
        /// Number of approvals required (quorum). 0 = all reviewers.
        pub quorum: u32,
        pub created_at: BlockNumberFor<T>,
        pub expires_at: Option<BlockNumberFor<T>>,
    }

    /// Individual reviewer decision.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct Decision<T: Config> {
        pub reviewer: T::AccountId,
        pub approved: bool,
        pub decided_at: BlockNumberFor<T>,
        pub reason_hash: Option<ContentHash>,
    }

    // ─── Storage ───────────────────────────────────────────────────

    #[pallet::storage]
    pub type NextApprovalId<T> = StorageValue<_, ApprovalId, ValueQuery>;

    #[pallet::storage]
    pub type Approvals<T: Config> =
        StorageMap<_, Blake2_128Concat, ApprovalId, ApprovalRecord<T>, OptionQuery>;

    /// Approvals by matter: MatterId × ApprovalId → ().
    #[pallet::storage]
    pub type ApprovalsByMatter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        ApprovalId,
        (),
        OptionQuery,
    >;

    /// Individual decisions: ApprovalId × ReviewerAccount → Decision.
    #[pallet::storage]
    pub type Decisions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ApprovalId,
        Blake2_128Concat,
        T::AccountId,
        Decision<T>,
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ApprovalRequested {
            approval_id: ApprovalId,
            matter_id: MatterId,
            subject_type: SubjectType,
            subject_id: u64,
            requester: T::AccountId,
        },
        ApprovalDecided {
            approval_id: ApprovalId,
            reviewer: T::AccountId,
            approved: bool,
        },
        ApprovalFinalized {
            approval_id: ApprovalId,
            status: ApprovalStatus,
        },
        ApprovalWithdrawn {
            approval_id: ApprovalId,
            by: T::AccountId,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        ApprovalNotFound,
        NotRequester,
        NotReviewer,
        AlreadyDecided,
        ApprovalNotPending,
        ApprovalExpired,
        NoReviewersProvided,
        InvalidQuorum,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request approval for a legal object.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn request_approval(
            origin: OriginFor<T>,
            matter_id: MatterId,
            subject_type: SubjectType,
            subject_id: u64,
            subject_hash: ContentHash,
            reviewers: BoundedVec<T::AccountId, T::MaxReviewers>,
            quorum: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!reviewers.is_empty(), Error::<T>::NoReviewersProvided);

            let effective_quorum = if quorum == 0 { reviewers.len() as u32 } else { quorum };
            ensure!(
                effective_quorum <= reviewers.len() as u32,
                Error::<T>::InvalidQuorum
            );

            let id = NextApprovalId::<T>::get();
            NextApprovalId::<T>::put(id.wrapping_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let expiry_blocks = T::DefaultExpiryBlocks::get();
            let expires_at = if expiry_blocks > BlockNumberFor::<T>::from(0u32) {
                Some(now + expiry_blocks)
            } else {
                None
            };

            let record = ApprovalRecord {
                id,
                matter_id,
                subject_type,
                subject_id,
                status: ApprovalStatus::Pending,
                subject_hash,
                requester: who.clone(),
                reviewers: reviewers.clone(),
                quorum: effective_quorum,
                created_at: now,
                expires_at,
            };

            Approvals::<T>::insert(id, &record);
            ApprovalsByMatter::<T>::insert(matter_id, id, ());

            T::AuditHook::on_state_change(
                Some(matter_id),
                &who,
                ActionType::Create,
                SubjectType::Approval,
                id,
                None,
                Some(subject_hash),
            );

            Self::deposit_event(Event::ApprovalRequested {
                approval_id: id,
                matter_id,
                subject_type,
                subject_id,
                requester: who,
            });

            Ok(())
        }

        /// Submit a reviewer decision (approve or reject).
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn decide(
            origin: OriginFor<T>,
            approval_id: ApprovalId,
            approved: bool,
            reason_hash: Option<ContentHash>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let record = Approvals::<T>::get(approval_id)
                .ok_or(Error::<T>::ApprovalNotFound)?;

            ensure!(record.status == ApprovalStatus::Pending, Error::<T>::ApprovalNotPending);

            // Check expiry.
            let now = <frame_system::Pallet<T>>::block_number();
            if let Some(expires_at) = record.expires_at {
                ensure!(now <= expires_at, Error::<T>::ApprovalExpired);
            }

            // Verify reviewer is in the set.
            ensure!(
                record.reviewers.iter().any(|r| r == &who),
                Error::<T>::NotReviewer
            );
            ensure!(
                !Decisions::<T>::contains_key(approval_id, &who),
                Error::<T>::AlreadyDecided
            );

            let decision = Decision {
                reviewer: who.clone(),
                approved,
                decided_at: now,
                reason_hash,
            };
            Decisions::<T>::insert(approval_id, &who, &decision);

            T::AuditHook::on_state_change(
                Some(record.matter_id),
                &who,
                if approved { ActionType::Approve } else { ActionType::Reject },
                SubjectType::Approval,
                approval_id,
                None,
                None,
            );

            Self::deposit_event(Event::ApprovalDecided {
                approval_id,
                reviewer: who,
                approved,
            });

            // Check if quorum reached.
            Self::check_finalization(approval_id, &record);

            Ok(())
        }

        /// Withdraw a pending approval request (requester only).
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn withdraw(
            origin: OriginFor<T>,
            approval_id: ApprovalId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut record = Approvals::<T>::get(approval_id)
                .ok_or(Error::<T>::ApprovalNotFound)?;

            ensure!(record.requester == who, Error::<T>::NotRequester);
            ensure!(record.status == ApprovalStatus::Pending, Error::<T>::ApprovalNotPending);

            record.status = ApprovalStatus::Withdrawn;
            Approvals::<T>::insert(approval_id, &record);

            T::AuditHook::on_state_change(
                Some(record.matter_id),
                &who,
                ActionType::StatusChange,
                SubjectType::Approval,
                approval_id,
                None,
                None,
            );

            Self::deposit_event(Event::ApprovalWithdrawn {
                approval_id,
                by: who,
            });

            Ok(())
        }
    }

    // ─── Internal ──────────────────────────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Check whether enough decisions exist to finalize the approval.
        fn check_finalization(approval_id: ApprovalId, record: &ApprovalRecord<T>) {
            let mut approve_count = 0u32;
            let mut reject_count = 0u32;
            let total = record.reviewers.len() as u32;

            for reviewer in record.reviewers.iter() {
                if let Some(d) = Decisions::<T>::get(approval_id, reviewer) {
                    if d.approved {
                        approve_count += 1;
                    } else {
                        reject_count += 1;
                    }
                }
            }

            let new_status = if approve_count >= record.quorum {
                Some(ApprovalStatus::Approved)
            } else if reject_count > total.saturating_sub(record.quorum) {
                // Impossible to reach quorum even if all remaining approve.
                Some(ApprovalStatus::Rejected)
            } else {
                None
            };

            if let Some(status) = new_status {
                Approvals::<T>::mutate(approval_id, |maybe_rec| {
                    if let Some(rec) = maybe_rec {
                        rec.status = status;
                    }
                });
                Self::deposit_event(Event::ApprovalFinalized {
                    approval_id,
                    status,
                });
            }
        }
    }
}
