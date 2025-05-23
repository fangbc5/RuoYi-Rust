//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.10

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ruoyi_common::utils::string::serialize_vec_u8_to_string;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_notice")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// 公告ID
    #[sea_orm(primary_key)]
    pub notice_id: i32,
    /// 公告标题
    pub notice_title: String,
    /// 公告类型
    pub notice_type: String,
    /// 公告内容
    #[sea_orm(column_type = "custom(\"longblob\")", nullable)]
    #[serde(serialize_with = "serialize_vec_u8_to_string")]
    pub notice_content: Option<Vec<u8>>,
    /// 公告状态
    pub status: Option<String>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime<Utc>>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime<Utc>>,
    /// 备注
    pub remark: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
