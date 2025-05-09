// ruoyi-system/src/entity/system/role_menu.rs
//! 角色菜单关联实体定义

use crate::entity::role::Entity as RoleEntity;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 角色菜单关联表
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_role_menu")]
pub struct Model {
    /// 角色ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i64,

    /// 菜单ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "RoleEntity",
        from = "Column::RoleId",
        to = "crate::entity::role::Column::RoleId"
    )]
    Role,
    #[sea_orm(
        belongs_to = "super::menu::Entity",
        from = "Column::MenuId",
        to = "super::menu::Column::MenuId"
    )]
    Menu,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<RoleEntity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}
