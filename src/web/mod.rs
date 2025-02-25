pub mod controllers;

use actix_web::{
  body::BoxBody, http::{
    header::{
      HeaderName, HeaderValue
    }, StatusCode
  }, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use crate::{
    config::CauthConfig,
    web::controllers::{
        ListPermissionsController,
        InsertPermissionController,
        DeletePermissionController,
        ListGroupsController,
        InsertGroupController,
        DeleteGroupController,
        GrantPermissionGroupController,
        RevokePermissionGroupController,
        InsertUserController,
        DeleteUserController,
        GetUserController,
        GetPermissionUserController,
        LoginUserController,
        LogoutUserController,
        GrantGroupUserController,
        RevokeGroupUserController,
        UserRegisterEventCreateController,
        UserRegisterEventCommitController,
        UserRegisterEventCancelController,
        UserLoginEventCreateController,
        UserLoginEventCancelController,
        UserLoginEventCommitController,
        UserDeleteEventCreateController,
        UserDeleteEventCommitController,
        UserDeleteEventCancelController,
    }
};

pub async fn run_server(config: CauthConfig) -> std::io::Result<()> {
    let binding = config.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(binding.clone()))
            .service(ListPermissionsController)
            .service(InsertPermissionController)
            .service(DeletePermissionController)
            .service(ListGroupsController)
            .service(InsertGroupController)
            .service(DeleteGroupController)
            .service(GrantPermissionGroupController)
            .service(RevokePermissionGroupController)
            .service(InsertUserController)
            .service(DeleteUserController)
            .service(GetUserController)
            .service(GetPermissionUserController)
            .service(LoginUserController)
            .service(LogoutUserController)
            .service(GrantGroupUserController)
            .service(RevokeGroupUserController)
            .service(UserRegisterEventCreateController)
            .service(UserRegisterEventCommitController)
            .service(UserRegisterEventCancelController)
            .service(UserLoginEventCreateController)
            .service(UserLoginEventCancelController)
            .service(UserLoginEventCommitController)
            .service(UserDeleteEventCreateController)
            .service(UserDeleteEventCommitController)
            .service(UserDeleteEventCancelController)
    })
    .bind(("127.0.0.1", config.port))?
    .run()
    .await?;

    return Ok(());
}

pub struct ServerResponse {
  status: StatusCode,
  body: Option<serde_json::Value>
}

impl ServerResponse {
  pub fn new(status: StatusCode, body: Option<serde_json::Value>) -> Self {
    return Self {
      status,
      body
    };
  }
}

impl Responder for ServerResponse {
  type Body = BoxBody;

  fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
    let mut response = HttpResponse::new(self.status);
    response.headers_mut()
      .insert(
        HeaderName::from_static("content-type"), 
        HeaderValue::from_static("application/json")
      );

    if let Some(body) = &self.body {
      let body = serde_json::to_string(body).unwrap();

      return response
        .set_body(BoxBody::new(body));
    } else {
      return response;
    }
  }
}
