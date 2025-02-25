#![allow(unused)]

use crate::{
    models::{Group, LoginSession, Order},
    util::{
        logging::{log_database_interaction, DatabaseOperationLogStatus},
        string::json_value_to_pretty_string,
    },
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{query, query_as, FromRow, PgConnection};
use std::error::Error;

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Permission {
    pub name: String,
    pub description: String,
}

impl ToString for Permission {
    fn to_string(&self) -> String {
        let formatted = json_value_to_pretty_string(&serde_json::to_value(&self).unwrap());

        return formatted;
    }
}

#[derive(Debug)]
pub enum PermissionListError {}

impl ToString for PermissionListError {
    fn to_string(&self) -> String {
        return "".to_string();
    }
}

#[derive(Debug)]
pub enum PermissionRetrieveError {
    /// Returned when a permission with specified name is not found
    NotFound,
}

impl ToString for PermissionRetrieveError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "Permission with this name cannot be found".to_string(),
        };
    }
}

#[derive(Debug)]
pub enum PermissionInsertError {
    /// Returned when the permission either has too long name or description
    /// or when a permission with provided name already exist
    NameError,
}

impl ToString for PermissionInsertError {
    fn to_string(&self) -> String {
        return match self {
      Self::NameError => "Either permission name or description is too long or permission with this name already exist.".to_string()
    };
    }
}

#[derive(Debug)]
pub enum PermissionDeleteError {
    /// Returned when the permission with specified name do not exist
    NotFound,
}

impl ToString for PermissionDeleteError {
    fn to_string(&self) -> String {
        return "".to_string();
    }
}

impl Permission {
    /// ## Permission::list
    ///
    /// Lists number of permissions in specified order with specified offset from the database
    ///
    pub async fn list(
        conn: &mut PgConnection,
        order: Option<Order>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, PermissionListError> {
        let order = order.unwrap_or(Order::Ascending);
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(10);

        let sql = format!(
            "SELECT * FROM permissions ORDER BY name {} OFFSET {} ROWS LIMIT {};",
            order.to_string(),
            offset,
            limit
        );
        let result = query_as(&sql).fetch_all(&mut *conn).await.unwrap();

        return Ok(result);
    }

    /// ## Permission::retrieve
    ///
    /// Retrieves a permission with specified name from the database
    ///
    /// Errors:
    /// + when permission with specified name do not exist
    ///
    pub async fn retrieve(
        conn: &mut PgConnection,
        name: &String,
    ) -> Result<Self, PermissionRetrieveError> {
        let sql = "SELECT * FROM permissions WHERE name = $1;";
        let result = query_as(&sql).bind(&name).fetch_one(&mut *conn).await;

        match result {
            Ok(result) => return Ok(result),
            Err(_) => return Err(PermissionRetrieveError::NotFound),
        };
    }

    /// ## Permission::insert
    ///
    /// Inserts a permission with provided data into the database <br>
    ///
    /// Errors:
    /// + when a permission with provided name already exist
    /// + when the name is longer than 255 chars or description is longer than 3000 chars
    ///
    pub async fn insert(
        conn: &mut PgConnection,
        name: &String,
        description: &String,
    ) -> Result<(), PermissionInsertError> {
        let sql = "INSERT INTO permissions (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(&mut *conn).await {
            Ok(_) => {
                log_database_interaction::<String>(
                    "Inserting permission into database.",
                    json!({ "name": name, "description": description }),
                    DatabaseOperationLogStatus::Ok,
                );
                return Ok(());
            }
            Err(err) => {
                log_database_interaction(
                    "Inserting permission into database.",
                    json!({ "name": name, "description": description }),
                    DatabaseOperationLogStatus::Err("Already exists"),
                );
                return Err(PermissionInsertError::NameError);
            }
        };
    }

    /// ## Permission::delete
    ///
    /// Deletes a permission with provided name from the database
    ///
    pub async fn delete(
        conn: &mut PgConnection,
        name: &String,
    ) -> Result<(), PermissionDeleteError> {
        let sql = "DELETE FROM permissions WHERE name = $1;";
        let result = query(&sql).bind(&name).execute(&mut *conn).await.unwrap();

        if result.rows_affected() > 0 {
            log_database_interaction::<String>(
                "Deleting permission from database.",
                json!({ "name": name }),
                DatabaseOperationLogStatus::Ok,
            );
            return Ok(());
        } else {
            log_database_interaction(
                "Deleting permission from database.",
                json!({ "name": name }),
                DatabaseOperationLogStatus::Err("Not found"),
            );
            return Err(PermissionDeleteError::NotFound);
        }
    }
}
