//! # Pallet Access Control
//!
//! Role-based and matter-scoped access control. Grants, revokes, and checks
//! permissions for accounts on specific matters or system-wide.
//! Every mutation emits an event and calls the cross-pallet `AuditHook`.

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

        /// Maximum number of permissions per grant.
        #[pallet::constant]
        type MaxPermissionsPerGrant: Get<u32>;
    }

    // ─── Permission Enum ───────────────────────────────────────────

    /// Granular permission flags for matter-scoped access.
    #[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    pub enum Permission {
        /// View matter and its linked objects.
        Read,
        /// Create evidence / documents under this matter.
        Write,
        /// Transition matter status.
        Manage,
        /// Request or grant approvals.
        Approve,
        /// Access audit trail.
        AuditRead,
        /// Admin: modify access grants.
        Admin,
    }

    // ─── Storage Types ─────────────────────────────────────────────

    /// An access grant: a set of permissions for one account on one matter.
    #[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct AccessGrant<T: Config> {
        pub grantee: T::AccountId,
        pub matter_id: MatterId,
        pub permissions: BoundedVec<Permission, T::MaxPermissionsPerGrant>,
        pub granted_by: T::AccountId,
        pub granted_at: BlockNumberFor<T>,
        pub active: bool,
    }

    // ─── Storage ───────────────────────────────────────────────────

    /// ACL: (MatterId, AccountId) → AccessGrant.
    #[pallet::storage]
    pub type Grants<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        T::AccountId,
        AccessGrant<T>,
        OptionQuery,
    >;

    /// Per-matter admin set for quick admin checks: MatterId → admin AccountId → ().
    #[pallet::storage]
    pub type MatterAdmins<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MatterId,
        Blake2_128Concat,
        T::AccountId,
        (),
        OptionQuery,
    >;

    // ─── Events ────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccessGranted {
            matter_id: MatterId,
            grantee: T::AccountId,
            granted_by: T::AccountId,
        },
        AccessRevoked {
            matter_id: MatterId,
            grantee: T::AccountId,
            revoked_by: T::AccountId,
        },
        AdminDesignated {
            matter_id: MatterId,
            admin: T::AccountId,
            designated_by: T::AccountId,
        },
        AdminRemoved {
            matter_id: MatterId,
            admin: T::AccountId,
            removed_by: T::AccountId,
        },
    }

    // ─── Errors ────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        NotAuthorized,
        GrantNotFound,
        AlreadyGranted,
        NoPermissionsSpecified,
        AlreadyAdmin,
        NotAdmin,
    }

    // ─── Extrinsics ────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Grant access permissions to an account on a matter.
        /// Caller must be a matter admin (or root/sudo).
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn grant_access(
            origin: OriginFor<T>,
            matter_id: MatterId,
            grantee: T::AccountId,
            permissions: BoundedVec<Permission, T::MaxPermissionsPerGrant>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!permissions.is_empty(), Error::<T>::NoPermissionsSpecified);
            ensure!(
                MatterAdmins::<T>::contains_key(matter_id, &who),
                Error::<T>::NotAuthorized
            );
            ensure!(
                !Grants::<T>::contains_key(matter_id, &grantee),
                Error::<T>::AlreadyGranted
            );

            let now = <frame_system::Pallet<T>>::block_number();
            let grant = AccessGrant {
                grantee: grantee.clone(),
                matter_id,
                permissions,
                granted_by: who.clone(),
                granted_at: now,
                active: true,
            };
            Grants::<T>::insert(matter_id, &grantee, &grant);

            T::AuditHook::on_state_change(
                Some(matter_id),
                &who,
                ActionType::Create,
                SubjectType::Matter,
                matter_id,
                None,
                None,
            );

            Self::deposit_event(Event::AccessGranted {
                matter_id,
                grantee,
                granted_by: who,
            });

            Ok(())
        }

        /// Revoke access for an account on a matter.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn revoke_access(
            origin: OriginFor<T>,
            matter_id: MatterId,
            grantee: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                MatterAdmins::<T>::contains_key(matter_id, &who),
                Error::<T>::NotAuthorized
            );

            let mut grant = Grants::<T>::get(matter_id, &grantee)
                .ok_or(Error::<T>::GrantNotFound)?;

            grant.active = false;
            Grants::<T>::insert(matter_id, &grantee, &grant);

            T::AuditHook::on_state_change(
                Some(matter_id),
                &who,
                ActionType::Revoke,
                SubjectType::Matter,
                matter_id,
                None,
                None,
            );

            Self::deposit_event(Event::AccessRevoked {
                matter_id,
                grantee,
                revoked_by: who,
            });

            Ok(())
        }

        /// Designate an account as a matter admin.
        /// The first admin for a matter can be set by anyone (bootstrapping).
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn designate_admin(
            origin: OriginFor<T>,
            matter_id: MatterId,
            admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !MatterAdmins::<T>::contains_key(matter_id, &admin),
                Error::<T>::AlreadyAdmin
            );

            // Allow bootstrap: if no admins exist yet, anyone can set the first.
            let has_existing_admin = MatterAdmins::<T>::iter_prefix(matter_id).next().is_some();
            if has_existing_admin {
                ensure!(
                    MatterAdmins::<T>::contains_key(matter_id, &who),
                    Error::<T>::NotAuthorized
                );
            }

            MatterAdmins::<T>::insert(matter_id, &admin, ());

            T::AuditHook::on_state_change(
                Some(matter_id),
                &who,
                ActionType::Create,
                SubjectType::Matter,
                matter_id,
                None,
                None,
            );

            Self::deposit_event(Event::AdminDesignated {
                matter_id,
                admin,
                designated_by: who,
            });

            Ok(())
        }

        /// Remove an account from matter admin set.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn remove_admin(
            origin: OriginFor<T>,
            matter_id: MatterId,
            admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                MatterAdmins::<T>::contains_key(matter_id, &who),
                Error::<T>::NotAuthorized
            );
            ensure!(
                MatterAdmins::<T>::contains_key(matter_id, &admin),
                Error::<T>::NotAdmin
            );

            MatterAdmins::<T>::remove(matter_id, &admin);

            T::AuditHook::on_state_change(
                Some(matter_id),
                &who,
                ActionType::Revoke,
                SubjectType::Matter,
                matter_id,
                None,
                None,
            );

            Self::deposit_event(Event::AdminRemoved {
                matter_id,
                admin,
                removed_by: who,
            });

            Ok(())
        }
    }

    // ─── Public API for other pallets ──────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Check if an account has a specific permission on a matter.
        pub fn has_permission(
            matter_id: MatterId,
            account: &T::AccountId,
            permission: Permission,
        ) -> bool {
            // Admins have all permissions.
            if MatterAdmins::<T>::contains_key(matter_id, account) {
                return true;
            }
            if let Some(grant) = Grants::<T>::get(matter_id, account) {
                grant.active && grant.permissions.contains(&permission)
            } else {
                false
            }
        }
    }
}
