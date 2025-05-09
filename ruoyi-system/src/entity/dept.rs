// ruoyi-system/src/entity/system/dept.rs
//! 部门实体定义

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 部门实体
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_dept")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// 部门ID
    #[sea_orm(primary_key)]
    pub dept_id: i64,

    /// 父部门ID
    pub parent_id: Option<i64>,

    /// 祖级列表
    pub ancestors: Option<String>,

    /// 部门名称
    pub dept_name: Option<String>,

    /// 显示顺序
    pub order_num: Option<i32>,

    /// 负责人
    pub leader: Option<String>,

    /// 联系电话
    pub phone: Option<String>,

    /// 邮箱
    pub email: Option<String>,

    /// 状态
    pub status: Option<String>,

    /// 删除标志
    pub del_flag: Option<String>,

    /// 创建者
    pub create_by: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime<Utc>>,

    /// 更新者
    pub update_by: Option<String>,

    /// 更新时间
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "Entity")]
    Child,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::DeptId"
    )]
    Parent,
    #[sea_orm(has_many = "super::user::Entity")]
    User,
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}