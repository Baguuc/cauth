pub mod permissions;
pub mod groups;
pub mod users;

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
        revoke_group::controller as RevokeGroupUserController,
        event::{
            register::{
                create::controller as UserRegisterEventCreateController,
                commit::controller as UserRegisterEventCommitController,
                cancel::controller as UserRegisterEventCancelController
            },
            login::{
                create::controller as UserLoginEventCreateController,
                commit::controller as UserLoginEventCommitController,
                cancel::controller as UserLoginEventCancelController
            },
            delete::{
                create::controller as UserDeleteEventCreateController,
                commit::controller as UserDeleteEventCommitController,
                cancel::controller as UserDeleteEventCancelController
            }
        }
    }
};
