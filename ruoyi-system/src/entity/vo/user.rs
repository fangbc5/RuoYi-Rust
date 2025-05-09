use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::prelude::*;

/// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    /// 管理员
    pub is_admin: bool,
    /// 用户ID
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
    pub sex: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 状态
    pub status: Option<String>,
    /// 删除标志
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
    /// 部门
    pub dept: Option<DeptModel>,
    /// 角色
    pub roles: Vec<RoleModel>,
}

impl UserInfo {
    /// 创建新用户
    pub fn new(
        user_id: i64,
        user_name: String,
        nick_name: String,
        password: String,
        status: String,
    ) -> Self {
        Self {
            is_admin: UserInfo::is_admin(user_id),
            user_id,
            dept_id: None,
            user_name,
            nick_name,
            user_type: None,
            email: None,
            phonenumber: None,
            sex: None,
            avatar: None,
            password: Some(password),
            status: Some(status),
            del_flag: None,
            login_ip: None,
            login_date: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: None,
            dept: None,
            roles: vec![],
        }
    }

    pub fn from_model(model: UserModel) -> Self {
        Self {
            is_admin: UserInfo::is_admin(model.user_id),
            user_id: model.user_id,
            dept_id: model.dept_id,
            user_name: model.user_name,
            nick_name: model.nick_name,
            user_type: model.user_type,
            email: model.email,
            phonenumber: model.phonenumber,
            sex: model.sex,
            avatar: model.avatar,
            password: model.password,
            status: model.status,
            del_flag: model.del_flag,
            login_ip: model.login_ip,
            login_date: model.login_date,
            create_by: model.create_by,
            create_time: model.create_time,
            update_by: model.update_by,
            update_time: model.update_time,
            remark: model.remark,
            dept: None,
            roles: vec![],
        }
    }

    /// 检查用户状态是否正常
    pub fn is_active(&self) -> bool {
        self.status == Some("0".to_string())
    }

    /// 是否是管理员
    pub fn is_admin(user_id: i64) -> bool {
        user_id == 1
    }
}

/// 用户信息（不包含敏感信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeUserInfo {
    /// 用户ID
    pub id: i64,
    /// 用户名
    pub username: String,
    /// 昵称
    pub nickname: String,
    /// 手机号码
    pub phone: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 性别（0-未知, 1-男, 2-女）
    pub sex: Option<String>,
    /// 状态（0-禁用, 1-正常）
    pub status: String,
    /// 部门ID
    pub dept_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime<Utc>>,
}
