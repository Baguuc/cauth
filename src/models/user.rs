use std::{error::Error, io::Read};

use argon2::{password_hash::{self, rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{prelude::FromRow, query, query_as, PgConnection, PgPool, Transaction};

use crate::util::string::json_value_to_pretty_string;

use crate::models::{event::{Event, EventType}, login_session::{LoginSession, LoginSessionStatus}, Order};

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct User {
    pub login: String,
    pub password_hash: String,
    pub details: Value
}

impl ToString for User {
    fn to_string(&self) -> String {
        let formatted = json_value_to_pretty_string(&serde_json::to_value(&self).unwrap());

        return formatted;
    }
}

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UserCredentials {
    pub login: String,
    pub password: String,
}

pub type UserListError = ();

#[derive(Debug)]
pub enum UserRetrieveError {
    /// Returned when a user with specified login is not found
    NotFound,
}

impl ToString for UserRetrieveError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "This user cannot be found".to_string()
        }
    }
}

#[derive(Debug)]
pub enum UserInsertError {
    /// Returned when the user either has too long login,
    /// a user with provided login already exist
    /// or one of the provided groups do not exist
    NameError,
    /// Returned when the provided password cannot be hashed
    CannotHash(String)
}

impl ToString for UserInsertError {
    fn to_string(&self) -> String {
        return match self {
            Self::NameError => "Either the provided login is too long, this user already exist or one of the provided groups do not exist.".to_string(),
            Self::CannotHash(err) => format!("Password hashing error: {}.", err)
        }
    }
}

pub type UserDeleteError = ();
pub enum UserHasPermissionError {
    /// Returned when the user do not have queried permissions
    Unauthorized
}

impl ToString for UserHasPermissionError {
    fn to_string(&self) -> String {
        return match self {
            Self::Unauthorized => "This user do not have this permission".to_string()
        };
    }
}

pub enum UserLoginError {
    /// Returned when the user is not found
    NotFound,
    /// Returned when the credentials are invalid
    InvalidCredentials
}

pub enum UserGrantError {
    /// Returned either when a user with provided login do not exist
    /// or provided group do not exist
    NameError
}

impl ToString for UserGrantError {
    fn to_string(&self) -> String {
        return match self {
            Self::NameError => "Provided user or group do not exist".to_string()
        };
    }
}

pub enum UserRevokeError {
    /// Returned either when a user with provided login do not exist
    /// or provided group do not exist
    NameError
}

impl ToString for UserRevokeError {
    fn to_string(&self) -> String {
        return match self {
            Self::NameError => "Provided user or group do not exist".to_string()
        };
    }
}

impl User {
    /// ## User::list
    /// 
    /// Lists number of users in specified order with specified offset from the database
    /// 
    pub async fn list(
        conn: &mut PgConnection,
        order: Option<Order>,
        offset: Option<usize>,
        limit: Option<usize>
    ) -> Result<Vec<Self>, UserListError> {
        let order = order.unwrap_or(Order::Ascending);
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(10);
        let sql = format!(
            "SELECT * FROM users ORDER BY {} OFFSET {} ROWS LIMIT {};",
            order.to_string(),
            offset,
            limit
        );
        let result = query_as(&sql)
            .fetch_all(&mut *conn)
            .await
            .unwrap();

        return Ok(result);
    }

    /// ## User::retrieve
    /// 
    /// Retrieves a user with specified name from the database
    /// 
    /// Errors:
    /// + when permission with specified name do not exist
    /// 
    pub async fn retrieve(
        conn: &mut PgConnection,
        login: &String
    ) -> Result<Self, UserRetrieveError> {
        let sql = "SELECT * FROM users WHERE login = $1;";
        let result = query_as(&sql)
            .bind(&login)
            .fetch_one(&mut *conn)
            .await;

        match result {
            Ok(result) => return Ok(result),
            Err(_) => return Err(UserRetrieveError::NotFound)
        };
    }

    /// ## User::insert
    /// 
    /// Inserts a user with provided data into the database <br>
    /// 
    /// Errors:
    /// + when a user with provided login already exist
    /// + when the login is longer than 255 chars
    /// 
    pub async fn insert(
        conn: &mut PgConnection,
        login: String,
        password: String,
        details: Value
    ) -> Result<(), UserInsertError> {
        let sql = "
            INSERT INTO
                users (login, password_hash, details)
            VALUES
                ($1, $2, $3)
            ;
        ";
       
        let password_hash = match hash_password(password) {
            Ok(hash) => hash,
            Err(err) => return Err(UserInsertError::CannotHash(err.to_string()))
        };

        let result = query(sql)
            .bind(&login)
            .bind(password_hash)
            .bind(&details)
            .execute(&mut *conn)
            .await;

        match result {
            Ok(_) => (),
            Err(_) => return Err(UserInsertError::NameError)
        };

        return Ok(());
    }


    /// ## User::delete
    /// 
    /// Deletes a user and all of it's related data from the database
    /// 
    pub async fn delete(
        conn: &mut PgConnection,
        login: String
    ) -> Result<(), UserDeleteError> {
        let sql = "DELETE FROM users WHERE login = $1";
        let q = query(sql)
            .bind(&login)
            .execute(&mut *conn)
            .await;

        let sql = "DELETE FROM users_groups WHERE user_login = $1";
        let q = query(sql)
            .bind(&login)
            .execute(&mut *conn)
            .await;

        let sql = "DELETE FROM login_sessions WHERE user_login = $1";
        let q = query(sql)
            .bind(&login)
            .execute(&mut *conn)
            .await;

        return Ok(());
    }

    /// ## User::has_permission
    /// 
    /// Check if a user has a specified permission
    /// 
    /// Errors:
    /// + When the user do not exist
    /// + When the credentials are invalid
    /// 
    pub async fn login(
        conn: &mut PgConnection,
        login: String,
        password: String,
        session_status: LoginSessionStatus
    ) -> Result<i64, UserLoginError> {
        let user = match Self::retrieve(conn, &login).await {
            Ok(user) => user,
            Err(_) => return Err(UserLoginError::NotFound)
        };

        let password_hash = &PasswordHash::parse(user.password_hash.as_str(), password_hash::Encoding::B64).unwrap();
        match Argon2::default().verify_password(password.as_bytes(), password_hash) {
            Ok(_) => (),
            Err(_) => return Err(UserLoginError::InvalidCredentials)
        };

        let session_id = LoginSession::insert(
            conn,
            login,
            session_status
        )
        .await
        .unwrap();

        return Ok(session_id);
    }

    /// ## User::has_permission
    /// 
    /// Check if a user has a specified permission
    /// 
    pub async fn has_permissions(
        self: &Self,
        conn: &mut PgConnection,
        permission_name: String
    ) -> bool {
        let sql = "
            SELECT
                gp.permission_name
            FROM
                users u
            INNER JOIN
                users_groups ug
            ON
                u.login = ug.user_login
            INNER JOIN
                groups_permissions gp
            ON
                ug.group_name = gp.group_name
            WHERE
                u.login = $1
            AND
                gp.permission_name  = $2
            LIMIT
                1;
        ";
        let q = query(sql)
            .bind(&self.login)
            .bind(&permission_name);
        let num_rows = q
            .execute(&mut *conn)
            .await
            .unwrap()
            .rows_affected();

        if num_rows == 0 {
            return false;
        }

        return true;
    }

    /// ## User::grant_group
    /// 
    /// Grants user a group with specified name
    /// 
    /// Errors:
    /// + When provided user or group do not exist
    /// 
    pub async fn grant_group(
        conn: &mut PgConnection,
        login: &String,
        group_name: &String
    ) -> Result<(), UserGrantError> {
        let sql = "INSERT INTO users_groups (user_login, group_name) VALUES ($1, $2);";
        let result = query(sql)
            .bind(login)
            .bind(group_name)
            .execute(&mut *conn)
            .await;

        match result {
            Ok(_) => (),
            Err(_) => return Err(UserGrantError::NameError)
        };

        return Ok(());
    }

    /// ## User::revoke_group
    /// 
    /// Revokes a group from user with specified login
    /// 
    /// Errors:
    /// + When provided user or group do not exist
    /// 
    pub async fn revoke_group(
        conn: &mut PgConnection,
        login: &String,
        group_name: &String
    ) -> Result<(), UserRevokeError> {
        let sql = "DELETE FROM users_groups WHERE user_login = $1 AND group_name = $2;";
        let result = query(sql)
            .bind(login)
            .bind(group_name)
            .execute(&mut *conn)
            .await
            .unwrap();

        if result.rows_affected() == 0 {
            return Err(UserRevokeError::NameError);
        }

        return Ok(());
    }


    /// ## User::event
    ///
    /// Get an UserEvent instance for user event creation
    ///
    pub fn event() -> UserEvent {
        return UserEvent;
    }
}


struct UserEvent;


impl UserEvent {
    /// ## UserEvent::register
    ///
    /// Insert a UserRegister event into database
    ///
    /// Errors:
    /// + The password cannot be hashed
    ///
    pub async fn register(
        conn: &mut PgConnection,
        login: String,
        password: String,
        details: serde_json::Value
    ) -> Result<(), UserInsertError> {
        let password_hash = match hash_password(password) {
            Ok(hash) => hash,
            Err(e) => return Err(UserInsertError::CannotHash(e.to_string()))
        };
        let data = User {
            login,
            password_hash,
            details
        };
        let data = serde_json::to_value(&data).unwrap();
        let _ = Event::insert(conn, EventType::UserRegister, data).await;
    
        return Ok(());
    }


    /// ## UserEvent::login
    ///
    /// Insert a UserLogin event into database
    ///
    /// Errors:
    /// + When the credentials are incorrect and the login session cannot be created
    ///
    pub async fn login(
        conn: &mut PgConnection,
        login: String,
        password: String
    ) -> Result<(), UserLoginError> {
        let session_id = User::login(conn, login, password, LoginSessionStatus::OnHold).await?;
        let data = serde_json::to_value(&session_id).unwrap();
        let _ = Event::insert(conn, EventType::UserLogin, data).await;
        
        return Ok(());
    }
     


    /// ## UserEvent::delete
    ///
    /// Insert a UserDelete event into database
    ///
    pub async fn delete(
        conn: &mut PgConnection,
        login: String
    ) {
        let data = serde_json::to_value(&login).unwrap();
        let _ = Event::insert(conn, EventType::UserDelete, data).await;
    }
}


fn hash_password(password: String) -> Result<String, String> {
    let pwd = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = match Argon2::default().hash_password(pwd, &salt) {
        Ok(hash) => hash,
        Err(err) => return Err(err.to_string())
    }
    .to_string();
    
    return Ok(password_hash);
}
