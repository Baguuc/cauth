pub mod subcommands;

use clap::{
  Parser,
  Subcommand
};
use crate::{cli::subcommands::{
  admin::AdminCommand, config::ConfigCommand, daemon::DaemonCommand, run::RunCommand
}, config::CauthConfig, models::{Group, Permission}};



#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CauthCli {
  #[clap(subcommand)]
  pub action: ActionType
}

impl CauthCli {
  pub fn run(self) {
    let _ = match self.action {
      ActionType::Run(cmd) => {
        let config = CauthConfig::parse_or_edit();

        cmd.run(config);
      },
      ActionType::Daemon(cmd) =>{
        let config = CauthConfig::parse_or_edit();

        cmd.run(config);
      },
      ActionType::Admin(cmd) => {
        let config = CauthConfig::parse_or_edit();

        cmd.run(config);
      },
      ActionType::Config(cmd) => {
        cmd.run();
      }
    };
  }
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
  Run(RunCommand),
  Daemon(DaemonCommand),
  Config(ConfigCommand),
  Admin(AdminCommand)
}


pub async fn init_defaults(config: &CauthConfig) {
  let mut tx = config.db_conn
    .begin()
    .await
    .unwrap();

  let _ = Permission::insert(
    &mut tx,
    &"cauth:permissions:get".to_string(), 
    &"permission to retrieve the permission list from the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:permissions:post".to_string(), 
    &"permission to post new permission to the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:permissions:delete".to_string(), 
    &"permission to delete a permission from the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:groups:get".to_string(), 
    &"permission to retrieve the groups list from the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:groups:post".to_string(), 
    &"permission to post new group to the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:groups:delete".to_string(), 
    &"permission to post new group to the database".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:groups:update".to_string(), 
    &"permission to grant/revoke permissions to groups".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:users:update".to_string(), 
    &"permission to grant/revoke groups to users".to_string()
  )
  .await;

  let _ = Permission::insert(
    &mut tx,
    &"cauth:users:delete".to_string(), 
    &"permission to delete ANY user on the service, use with caution.".to_string()
  )
  .await;
  
  let _ = Group::insert(
    &mut tx,
    &"root".to_string(), 
    &"the most privileged group, having to permissions to do everything. Caution: do not grant this group to any untrusted user as it can result in damages done to your system. Instead, create their own group fitting their needs.".to_string(),
    &vec![
      "cauth:permissions:get".to_string(),
      "cauth:permissions:post".to_string(),
      "cauth:permissions:delete".to_string(),
      "cauth:groups:get".to_string(),
      "cauth:groups:post".to_string(),
      "cauth:groups:delete".to_string(),
      "cauth:groups:update".to_string(),
      "cauth:users:update".to_string(),
      "cauth:users:delete".to_string()
    ]
  )
  .await;

  let _ = tx.commit().await;
}
