//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.10

use actix_web::HttpRequest;
use chrono::{DateTime, Utc};
use ruoyi_common::utils::{http, ip};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_login_info")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub info_id: i64,
    pub user_name: Option<String>,
    pub ipaddr: Option<String>,
    pub login_location: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub status: Option<String>,
    pub msg: Option<String>,
    pub login_time: Option<DateTime<Utc>>,
}

impl Model {
    pub fn from_request(request: &HttpRequest, user_name: &str, msg: &str) -> Self {
        let ipaddr = ip::get_real_ip_by_request(request);
        Self {
            info_id: 0,
            user_name: Some(user_name.to_string()),
            ipaddr: Some(ipaddr.clone()),
            login_location: Some(ip::get_ip_location(&ipaddr)),
            browser: Some(http::get_browser_info(&request)),
            os: Some(http::get_os_info(&request)),
            status: Some("".to_string()),
            msg: Some(msg.to_string()),
            login_time: Some(Utc::now()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
