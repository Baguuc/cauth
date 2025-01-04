use actix_web::{
  delete, 
  get, 
  http::StatusCode, 
  post, 
  web::{
    Data, 
    Json,
    Query
  }, Responder
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgConnection;
use crate::{
  models::{
    group::{
      Group,
      GroupInsertError,
      GroupDeleteError
    },
    LoginSession,
    Order
  },
  config::CauthConfig,
  web::ServerResponse
};

#[derive(Deserialize)]
struct GetGroupsQueryData {
    session_token: String,
    order_in: Option<Order>,
    page: Option<usize>
}

#[get("/groups")]
pub async fn get_groups(
  query: Query<GetGroupsQueryData>,
  data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    "groups:get"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();
  
  let result = Group::list(
    &mut db_conn,
    query.order_in,
    Some(query.page.unwrap_or(0) * 10),
    Some(10)
  )
  .await
  .unwrap();

  return ServerResponse::new(
    StatusCode::OK,
    Some(json!(result))
  );
}

#[derive(Deserialize)]
struct PostGroupQueryData {
  session_token: String,
  auto_commit: Option<bool>
}

#[derive(Deserialize)]
struct PostGroupJsonData {
  name: String,
  description: String,
  permissions: Vec<String>
}

#[post("/groups")]
pub async fn post_group(
  query: Query<PostGroupQueryData>,
  json: Json<PostGroupJsonData>,
  data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let auto_commit = query
    .auto_commit
    .unwrap_or(true);

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    "groups:post"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if let Err(_) = insert_group(
    &mut db_conn,
    &json.name,
    &json.description,
    &json.permissions,
    auto_commit
  ).await {
    return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    );
  }

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}


async fn insert_group(
  conn: &mut PgConnection, 
  name: &String, 
  description: &String,
  permissions: &Vec<String>,
  auto_commit: bool
) -> Result<(), GroupInsertError> {
  if auto_commit {
    Group::insert(
      conn,
      name,
      description,
      permissions
    )
    .await?;
  } else {
    Group::event().insert(
      conn,
      name,
      description,
      permissions
    )
    .await;
  }

  return Ok(());
}
