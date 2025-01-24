pub mod permissions;
pub mod groups;
pub mod users;
pub mod events;

pub use self::{
    permissions::{
        list::controller as ListPermissionsController,
        insert::controller as InsertPermissionController,
        delete::controller as DeletePermissionController
    },
    groups::{
        list::controller as ListGroupsController,
        insert::controller as InsertGroupController,
        delete::controller as DeleteGroupController,
        grant_permission::controller as GrantPermissionGroupController,
        revoke_permission::controller as RevokePermissionGroupController
    },
    users::{
        insert::controller as InsertUserController,
        delete::controller as DeleteUserController,
        get::controller as GetUserController,
        get_permission::controller as GetPermissionUserController,
        login::controller as LoginUserController,
        logout::controller as LogoutUserController,
        grant_group::controller as GrantGroupUserController,
        revoke_group::controller as RevokeGroupUserController
    },
    events::{
        commit::controller as CommitEventController
    }
};
