// ruoyi-system/src/entity/system/menu.rs
//! 菜单实体定义

use chrono::{DateTime, Utc};
use ruoyi_common::utils::string::serialize_i32_to_string;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
/// 系统菜单实体
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_menu")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// 菜单ID
    #[sea_orm(primary_key)]
    pub menu_id: i64,
    /// 菜单名称
    pub menu_name: String,
    /// 父菜单ID
    pub parent_id: Option<i64>,
    /// 显示顺序
    pub order_num: Option<i32>,
    /// 路由地址
    pub path: Option<String>,
    /// 组件路径
    pub component: Option<String>,
    /// 路由参数
    pub query: Option<String>,
    /// 路由名称
    pub route_name: Option<String>,
    /// 是否为外链（0否 1是）
    #[serde(serialize_with = "serialize_i32_to_string")]
    pub is_frame: Option<i32>,
    /// 是否缓存（0否 1是）
    #[serde(serialize_with = "serialize_i32_to_string")]
    pub is_cache: Option<i32>,
    /// 菜单类型（M目录 C菜单 F按钮）
    #[sea_orm(column_type = "Char(Some(1))")]
    pub menu_type: Option<String>,
    /// 是否显示（0不显示 1显示）
    #[sea_orm(column_type = "Char(Some(1))")]
    pub visible: Option<String>,
    /// 状态（0停用 1正常）
    #[sea_orm(column_type = "Char(Some(1))")]
    pub status: Option<String>,
    /// 权限标识
    pub perms: Option<String>,
    /// 菜单图标
    pub icon: Option<String>,
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
pub enum Relation {
    #[sea_orm(has_many = "Entity")]
    Child,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::MenuId"
    )]
    Parent,
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
