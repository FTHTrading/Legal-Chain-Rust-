//! # Pallet Agent Policy
//!
//! AI agent registration, scoped permission policies, and usage tracking.
//! Agents operate under bounded policies: which matters they can touch,
//! what actions they may perform, and per-block rate limits.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use legal_chain_common_types::{
        ActionType, AuditHook, MatterId, SubjectType,
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

        /// Maximum number of matter-scope entries an agent can be bound to.
        #[pallet::constant]
        type MaxScopeEntries: Get<u32>;

        /// Maximum actions per block for any single agent.
        #[pallet::constant]
        type MaxActionsPerBlock: Get<u32>;
    }

    // ─── Types ─────────────────────────────────────────────────────

    /// What an agent is permitted to do.
    #[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    pub enum AgentCapability {
        ReadEvidence,
        ReadDocuments,
        WriteDocuments,
        SubmitEvidence,
        GenerateSummary,
        RequestApproval,
        AuditRead,
    }

    /// Scope of an agent's access.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, PartialEq, Eq)]
    pub enum PolicyScope {
        /// Access across all matters (system-level agent).
        Global,
        /// Bound to specific matters.
        MatterScoped(MatterId),
    }

    pub type AgentId = u64;

    /// On-chain record for a registered AI agent.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct AgentRecord<T: Config> {
        pub id: AgentId,
        pub controller: T::AccountId,
        pub scope: PolicyScope,
        pub capabilities: BoundedVec<AgentCapability, T::MaxScopeEntries>,
        pub rate_limit: u32,
        pub active: bool,
        pub registered_at: BlockNumberFor<T>,
    }

    // ─── Storage ───────────────────────────────────────────────────

    /// Auto-incrementing agent id counter.
    #[pallet::storage]
    pub type NextAgentId<T: Config> = StorageValue<_, AgentId, ValueQuery>;

    /// Agent record by id.
    #[pallet::storage]
    pub type Agents<T: Config> =
        StorageMap<_, Blake2_128Concat, AgentId, AgentRecord<T>, OptionQuery>;

    /// Index: controller account → agent id (1 agent per controller for simplicity).
    #[pallet::storage]
    pub type AgentByController<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AgentId, OptionQuery>;

    /// Per-block usage counter: (AgentId, BlockNumber) → actions taken.
    #[pallet::storage]
    pub type UsageCounter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AgentId,
        Blake2_128Concat,
        BlockNumberFor<T>,
        u32,
        ValueQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AgentRegistered {
            agent_id: AgentId,
            controller: T::AccountId,
        },
        AgentRevoked {
            agent_id: AgentId,
            revoked_by: T::AccountId,
        },
        PolicyUpdated {
            agent_id: AgentId,
            updated_by: T::AccountId,
        },
        UsageRecorded {
            agent_id: AgentId,
            block: BlockNumberFor<T>,
            count: u32,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        AgentNotFound,
        NotController,
        AlreadyRegistered,
        AgentInactive,
        RateLimitExceeded,
        NoCapabilitiesSpecified,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new AI agent with a scoped policy.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn register_agent(
            origin: OriginFor<T>,
            scope: PolicyScope,
            capabilities: BoundedVec<AgentCapability, T::MaxScopeEntries>,
            rate_limit: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !AgentByController::<T>::contains_key(&who),
                Error::<T>::AlreadyRegistered
            );
            ensure!(!capabilities.is_empty(), Error::<T>::NoCapabilitiesSpecified);

            let id = NextAgentId::<T>::get();
            NextAgentId::<T>::put(id.saturating_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let record = AgentRecord {
                id,
                controller: who.clone(),
                scope,
                capabilities,
                rate_limit,
                active: true,
                registered_at: now,
            };

            Agents::<T>::insert(id, &record);
            AgentByController::<T>::insert(&who, id);

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Register,
                SubjectType::AgentPolicy,
                id,
                None,
                None,
            );

            Self::deposit_event(Event::AgentRegistered {
                agent_id: id,
                controller: who,
            });

            Ok(())
        }

        /// Update an agent's capabilities and rate limit.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn update_policy(
            origin: OriginFor<T>,
            agent_id: AgentId,
            capabilities: BoundedVec<AgentCapability, T::MaxScopeEntries>,
            rate_limit: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut agent = Agents::<T>::get(agent_id).ok_or(Error::<T>::AgentNotFound)?;
            ensure!(agent.controller == who, Error::<T>::NotController);
            ensure!(agent.active, Error::<T>::AgentInactive);
            ensure!(!capabilities.is_empty(), Error::<T>::NoCapabilitiesSpecified);

            agent.capabilities = capabilities;
            agent.rate_limit = rate_limit;
            Agents::<T>::insert(agent_id, &agent);

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Update,
                SubjectType::AgentPolicy,
                agent_id,
                None,
                None,
            );

            Self::deposit_event(Event::PolicyUpdated {
                agent_id,
                updated_by: who,
            });

            Ok(())
        }

        /// Revoke an agent. Only the controller may do this.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn revoke_agent(
            origin: OriginFor<T>,
            agent_id: AgentId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut agent = Agents::<T>::get(agent_id).ok_or(Error::<T>::AgentNotFound)?;
            ensure!(agent.controller == who, Error::<T>::NotController);

            agent.active = false;
            Agents::<T>::insert(agent_id, &agent);

            T::AuditHook::on_state_change(
                None,
                &who,
                ActionType::Revoke,
                SubjectType::AgentPolicy,
                agent_id,
                None,
                None,
            );

            Self::deposit_event(Event::AgentRevoked {
                agent_id,
                revoked_by: who,
            });

            Ok(())
        }

        /// Record one usage tick for an agent (called by other pallets / off-chain workers).
        /// Reverts if the per-block rate limit is exceeded.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn bump_usage(
            origin: OriginFor<T>,
            agent_id: AgentId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let agent = Agents::<T>::get(agent_id).ok_or(Error::<T>::AgentNotFound)?;
            ensure!(agent.active, Error::<T>::AgentInactive);
            ensure!(agent.controller == who, Error::<T>::NotController);

            let now = <frame_system::Pallet<T>>::block_number();
            let current = UsageCounter::<T>::get(agent_id, now);
            ensure!(current < agent.rate_limit, Error::<T>::RateLimitExceeded);

            let new_count = current.saturating_add(1);
            UsageCounter::<T>::insert(agent_id, now, new_count);

            Self::deposit_event(Event::UsageRecorded {
                agent_id,
                block: now,
                count: new_count,
            });

            Ok(())
        }
    }
}
