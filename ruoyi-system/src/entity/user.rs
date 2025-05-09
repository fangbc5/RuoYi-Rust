// ruoyi-system/src/entity/user.rs
//! 系统用户实体定义

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 系统用户实体
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sys_user")]
pub struct Model {
    /// 用户ID
    #[sea_orm(primary_key)]
    pub user_id: i64,
    /// 部门ID
    pub dept_id: Option<i64>,
    /// 用户账号
    pub user_name: String,
    /// 用户昵称
    pub nick_name: String,
    /// 用户类型
    pub user_type: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机号码
    pub phonenumber: Option<String>,
    /// 性别
    #[sea_orm(column_type = "Char(Some(1))")]
    pub sex: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 状态
    #[sea_orm(column_type = "Char(Some(1))")]
    pub status: Option<String>,
    /// 删除标志
    #[sea_orm(column_type = "Char(Some(1))")]
    pub del_flag: Option<String>,
    /// 最后登录IP
    pub login_ip: Option<String>,
    /// 最后登录时间
    pub login_date: Option<DateTime<Utc>>,
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
    #[sea_orm(
        belongs_to = "super::dept::Entity",
        from = "Column::DeptId",
        to = "super::dept::Column::DeptId"
    )]
    Dept,
    #[sea_orm(
        has_many = "super::user_role::Entity",
        from = "Column::UserId",
        to = "super::user_role::Column::UserId"
    )]
    UserRole,
}

impl Related<super::dept::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dept.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
