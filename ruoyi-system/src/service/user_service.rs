// ruoyi-system/src/service/user_service.rs
//! 用户服务

use async_trait::async_trait;
use log::error;
use ruoyi_common::error::Error;
use ruoyi_common::utils::password::encrypt_password;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use sea_orm::{IntoActiveModel, Set};
use std::sync::Arc;

use crate::controller::user_controller::{CreateOrUpdateUserRequest, UserQuery};
use crate::entity::prelude::*;
use crate::entity::vo::user::UserInfo;
use crate::repository::dept_repository::DeptRepository;
use crate::repository::menu_repository::MenuRepository;
use crate::repository::role_repository::RoleRepository;
use crate::repository::user_repository::UserRepository;

/// 用户服务接口
#[async_trait]
pub trait UserService: Send + Sync {
    /// 获取用户列表
    async fn get_user_list(
        &self,
        query: UserQuery,
        page_param: PageParam,
    ) -> Result<(Vec<UserInfo>, u64)>;

    /// 获取用户详情
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserInfo>>;

    /// 通过用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> Result<Option<UserInfo>>;

    /// 创建用户
    async fn create_user(&self, req: CreateOrUpdateUserRequest) -> Result<UserModel>;

    /// 更新用户
    async fn update_user(&self, req: CreateOrUpdateUserRequest) -> Result<UserInfo>;

    /// 删除用户
    async fn delete_user_by_ids(&self, user_ids: Vec<i64>) -> Result<()>;

    /// 重置密码
    async fn reset_password(&self, user_id: i64, new_password: &str) -> Result<()>;

    /// 修改用户状态
    async fn update_user_status(&self, user_id: i64, status: &str) -> Result<()>;

    /// 检查用户是否存在
    async fn check_user_name_unique(&self, username: &str, user_id: Option<i64>) -> Result<bool>;

    /// 获取用户角色列表
    async fn get_user_roles(&self, user_id: i64) -> Result<Vec<RoleModel>>;

    /// 获取用户权限列表
    async fn get_user_permissions(&self, user_id: i64) -> Result<Vec<String>>;
}

/// 用户服务实现
pub struct UserServiceImpl {
    user_repository: Arc<dyn UserRepository>,
    role_repository: Arc<dyn RoleRepository>,
    menu_repository: Arc<dyn MenuRepository>,
    dept_repository: Arc<dyn DeptRepository>,
}

impl UserServiceImpl {
    /// 创建用户服务
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        role_repository: Arc<dyn RoleRepository>,
        menu_repository: Arc<dyn MenuRepository>,
        dept_repository: Arc<dyn DeptRepository>,
    ) -> Self {
        Self {
            user_repository,
            role_repository,
            menu_repository,
            dept_repository,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_user_list(
        &self,
        query: UserQuery,
        page_param: PageParam,
    ) -> Result<(Vec<UserInfo>, u64)> {
        // 分页查询用户（已经包含了部门关联查询）
        let (user_dept_pairs, total) = self
            .user_repository
            .find_user_list(&query, &page_param)
            .await?;

        // 将UserModel和DeptModel转换为UserInfo
        let user_infos = user_dept_pairs
            .into_iter()
            .map(|(user_model, dept_model)| {
                let mut user_info = UserInfo::from_model(&user_model);
                user_info.dept = dept_model;
                user_info
            })
            .collect();

        Ok((user_infos, total))
    }

    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserInfo>> {
        // 调用用户仓库查询用户
        let user_model = self.user_repository.find_by_id(user_id as i64).await?;
        if user_model.is_none() {
            error!("用户不存在: {}", user_id);
            return Ok(None);
        }
        let user_model = user_model.unwrap();
        let mut user = UserInfo::from_model(&user_model);
        // 获取部门信息
        if let Some(dept_id) = user_model.dept_id {
            user.dept = self.dept_repository.find_by_id(dept_id).await?;
        }
        // 获取角色信息
        user.roles = self
            .role_repository
            .get_roles_by_user_id(user_id as i64)
            .await?;
        Ok(Some(user))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<UserInfo>> {
        let user_model = self.user_repository.find_by_username(username).await?;
        if user_model.is_none() {
            error!("用户不存在: {}", username);
            return Ok(None);
        }
        let user_model = user_model.unwrap();
        let mut user = UserInfo::from_model(&user_model);
        // 获取部门信息
        if let Some(dept_id) = user_model.dept_id {
            user.dept = self.dept_repository.find_by_id(dept_id).await?;
        }
        Ok(Some(user))
    }

    async fn create_user(&self, req: CreateOrUpdateUserRequest) -> Result<UserModel> {
        // 用户密码进行编码
        let password = if let Some(password) = req.password {
            Some(encrypt_password(&password)?)
        } else {
            None
        };

        // 调用用户仓库创建用户
        let user_model = UserModel {
            user_id: 0,
            dept_id: req.dept_id,
            user_name: req.user_name.unwrap(),
            nick_name: req.nick_name.unwrap(),
            user_type: None,
            email: req.email,
            phonenumber: req.phonenumber,
            sex: req.sex,
            avatar: None,
            password: password,
            status: req.status,
            del_flag: None,
            login_ip: None,
            login_date: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: req.remark,
        };

        let user = self
            .user_repository
            .create_user(user_model.into_active_model(), req.role_ids, req.post_ids)
            .await?;
        Ok(user)
    }

    async fn update_user(&self, req: CreateOrUpdateUserRequest) -> Result<UserInfo> {
        // 先检查用户是否存在
        let user = self
            .user_repository
            .find_by_id(req.user_id.unwrap())
            .await?;
        let mut update_flag = false;
        if let Some(user) = user {
            let mut user_active_model = user.clone().into_active_model();
            if let Some(user_name) = req.user_name {
                user_active_model.user_name = Set(user_name);
                update_flag = true;
            }
            if let Some(nick_name) = req.nick_name {
                user_active_model.nick_name = Set(nick_name);
                update_flag = true;
            }
            if let Some(email) = req.email {
                user_active_model.email = Set(Some(email));
                update_flag = true;
            }
            if let Some(phonenumber) = req.phonenumber {
                user_active_model.phonenumber = Set(Some(phonenumber));
                update_flag = true;
            }
            if let Some(sex) = req.sex {
                user_active_model.sex = Set(Some(sex));
                update_flag = true;
            }
            if let Some(status) = req.status {
                user_active_model.status = Set(Some(status));
                update_flag = true;
            }
            if let Some(remark) = req.remark {
                user_active_model.remark = Set(Some(remark));
                update_flag = true;
            }
            if let Some(dept_id) = req.dept_id {
                user_active_model.dept_id = Set(Some(dept_id));
                update_flag = true;
            }
            if update_flag {
                let user_model = self
                    .user_repository
                    .update_user(user_active_model, req.role_ids, req.post_ids)
                    .await?;
                Ok(UserInfo::from_model(&user_model))
            } else {
                Ok(UserInfo::from_model(&user))
            }
        } else {
            Err(Error::NotFound(format!(
                "用户不存在: {}",
                req.user_id.unwrap()
            )))
        }
    }

    async fn delete_user_by_ids(&self, user_ids: Vec<i64>) -> Result<()> {
        // 模拟删除用户
        self.user_repository.delete_users_by_ids(user_ids).await?;
        Ok(())
    }

    async fn reset_password(&self, user_id: i64, new_password: &str) -> Result<()> {
        // 加密密码
        match encrypt_password(new_password) {
            Ok(bcrypt_password) => {
                self.user_repository
                    .reset_password(user_id, &bcrypt_password)
                    .await?;
            }
            Err(e) => return Err(Error::PasswordError(format!("加密密码失败: {}", e))),
        }
        Ok(())
    }

    async fn update_user_status(&self, user_id: i64, status: &str) -> Result<()> {
        self.user_repository
            .update_user_status(user_id, status)
            .await?;
        Ok(())
    }

    async fn check_user_name_unique(&self, username: &str, user_id: Option<i64>) -> Result<bool> {
        // 调用用户仓库检查用户是否存在
        self.user_repository
            .check_user_name_unique(username, user_id)
            .await
    }

    async fn get_user_roles(&self, user_id: i64) -> Result<Vec<RoleModel>> {
        // 调用仓库方法获取用户角色
        match self
            .role_repository
            .get_roles_by_user_id(user_id as i64)
            .await
        {
            Ok(roles) => Ok(roles),
            Err(e) => {
                error!("获取用户角色失败: {}", e);
                Err(Error::InternalServerError(format!(
                    "获取用户角色失败: {}",
                    e
                )))
            }
        }
    }

    async fn get_user_permissions(&self, user_id: i64) -> Result<Vec<String>> {
        // 在若依框架中，超级管理员直接返回所有权限
        match self
            .menu_repository
            .select_menus_by_user_id(user_id as i64)
            .await
        {
            Ok(menus) => {
                let permissions = menus
                    .iter()
                    .filter(|m| m.perms.is_some())
                    .map(|menu| menu.perms.clone().unwrap())
                    .collect();
                Ok(permissions)
            }
            Err(e) => {
                error!("获取用户权限失败: {}", e);
                Err(Error::InternalServerError(format!(
                    "获取用户权限失败: {}",
                    e
                )))
            }
        }
    }
}
