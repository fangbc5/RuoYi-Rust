// ruoyi-system/src/entity/system/user_role.rs
//! 用户角色关联实体定义

use crate::entity::role::Entity as RoleEntity;
use crate::entity::user::Entity as UserEntity;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 用户角色关联表
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_user_role")]
pub struct Model {
    /// 用户ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,

    /// 角色ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::UserId"
    )]
    User,
    #[sea_orm(
        belongs_to = "RoleEntity",
        from = "Column::RoleId",
        to = "crate::entity::role::Column::RoleId"
    )]
    Role,
}

impl ActiveModelBehavior for ActiveModel {}

// 为 Entity 添加关系实现
impl Related<UserEntity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<RoleEntity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}
