// ruoyi-system/src/entity/role.rs
//! 系统角色实体定义

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 系统角色实体
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_role")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// 角色ID
    #[sea_orm(primary_key)]
    pub role_id: i64,
    /// 角色名称
    pub role_name: String,
    /// 角色权限字符串
    pub role_key: String,
    /// 显示顺序
    pub role_sort: i32,
    /// 数据范围
    #[sea_orm(column_type = "Char(Some(1))")]
    pub data_scope: Option<String>,
    /// 菜单树选择项是否关联显示
    pub menu_check_strictly: Option<bool>,
    /// 部门树选择项是否关联显示
    pub dept_check_strictly: Option<bool>,
    /// 角色状态
    #[sea_orm(column_type = "Char(Some(1))")]
    pub status: String,
    /// 删除标志
    #[sea_orm(column_type = "Char(Some(1))")]
    pub del_flag: Option<String>,
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
    #[sea_orm(has_many = "crate::entity::role_menu::Entity")]
    RoleMenu,
}

impl Related<crate::entity::role_menu::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleMenu.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
