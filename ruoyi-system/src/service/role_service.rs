// ruoyi-system/src/service/role_service.rs
//! 角色服务

use async_trait::async_trait;
use log::error;
use ruoyi_common::error::Error::BusinessError;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};
use std::sync::Arc;

use crate::controller::role_controller::{CreateOrUpdateRoleRequest, RoleAuthUserQuery, RoleQuery};
use crate::entity::prelude::*;
use crate::entity::vo::user::UserInfo;
use crate::repository::role_repository::RoleRepository;

/// 角色服务接口
#[async_trait]
pub trait RoleService: Send + Sync {
    /// 获取角色列表
    async fn get_role_list(
        &self,
        query: RoleQuery,
        page_param: PageParam,
    ) -> Result<(Vec<RoleModel>, u64)>;

    /// 根据用户ID查询角色列表
    async fn get_roles_by_user_id(&self, user_id: i64) -> Result<Vec<RoleModel>>;

    /// 获取角色详情
    async fn get_role_by_id(&self, role_id: i64) -> Result<Option<RoleModel>>;

    /// 创建角色
    async fn create_role(&self, role: CreateOrUpdateRoleRequest) -> Result<RoleModel>;

    /// 更新角色
    async fn update_role(&self, role: CreateOrUpdateRoleRequest) -> Result<RoleModel>;

    /// 删除角色
    async fn delete_roles(&self, role_ids: Vec<i64>) -> Result<()>;

    /// 检查角色名称是否唯一
    async fn check_role_name_unique(&self, role_name: &str, role_id: Option<i64>) -> Result<bool>;

    /// 检查角色权限是否唯一
    async fn check_role_key_unique(&self, role_key: &str, role_id: Option<i64>) -> Result<bool>;

    /// 修改角色状态
    async fn change_role_status(&self, role_id: i64, status: &str) -> Result<RoleModel>;

    /// 获取角色分配的用户列表
    async fn get_role_allocated_users(
        &self,
        req: RoleAuthUserQuery,
    ) -> Result<(Vec<UserInfo>, u64)>;

    /// 获取角色未分配的用户列表
    async fn get_role_unallocated_users(
        &self,
        req: RoleAuthUserQuery,
    ) -> Result<(Vec<UserInfo>, u64)>;

    /// 批量授权用户角色
    async fn auth_role_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()>;

    /// 取消用户角色授权
    async fn cancel_auth_user(&self, role_id: i64, user_id: i64) -> Result<()>;

    /// 批量取消用户角色授权
    async fn cancel_auth_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()>;

    /// 获取所有角色
    async fn get_roles_all(&self) -> Vec<RoleModel>;
}

/// 角色服务实现
pub struct RoleServiceImpl {
    role_repository: Arc<dyn RoleRepository>,
}

impl RoleServiceImpl {
    /// 创建角色服务
    pub fn new(role_repository: Arc<dyn RoleRepository>) -> Self {
        Self { role_repository }
    }

    /// 构建查询条件
    fn build_query_condition(&self, query: &RoleQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        let mut has_condition = false;

        if let Some(name) = &query.role_name {
            if !name.is_empty() {
                condition = condition.add(RoleColumn::RoleName.contains(name));
                has_condition = true;
            }
        }

        if let Some(key) = &query.role_key {
            if !key.is_empty() {
                condition = condition.add(RoleColumn::RoleKey.contains(key));
                has_condition = true;
            }
        }

        if let Some(status) = &query.status {
            if !status.is_empty() {
                condition = condition.add(RoleColumn::Status.eq(status));
                has_condition = true;
            }
        }

        if let Some(begin_time) = query.begin_time {
            condition = condition.add(RoleColumn::CreateTime.gt(begin_time));
            has_condition = true;
        }

        if let Some(end_time) = query.end_time {
            condition = condition.add(RoleColumn::CreateTime.lt(end_time));
            has_condition = true;
        }

        if has_condition {
            Some(condition)
        } else {
            None
        }
    }

    fn build_allocated_users_condition(&self, req: &RoleAuthUserQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(user_name) = &req.user_name {
            condition = condition.add(UserColumn::UserName.contains(user_name));
        }
        if let Some(phonenumber) = &req.phonenumber {
            condition = condition.add(UserColumn::Phonenumber.contains(phonenumber));
        }
        Some(condition)
    }
}

#[async_trait]
impl RoleService for RoleServiceImpl {
    async fn get_role_list(
        &self,
        query: RoleQuery,
        page_param: PageParam,
    ) -> Result<(Vec<RoleModel>, u64)> {
        let condition = self.build_query_condition(&query);
        let (roles, total) = self
            .role_repository
            .find_list(condition, &page_param)
            .await?;
        Ok((roles, total))
    }

    async fn get_roles_by_user_id(&self, user_id: i64) -> Result<Vec<RoleModel>> {
        let roles = self.role_repository.get_roles_by_user_id(user_id).await?;
        Ok(roles)
    }

    async fn get_role_by_id(&self, role_id: i64) -> Result<Option<RoleModel>> {
        Ok(self.role_repository.find_by_id(role_id).await?)
    }

    async fn create_role(&self, req: CreateOrUpdateRoleRequest) -> Result<RoleModel> {
        let role_model = RoleModel {
            role_id: 0,
            role_name: req.role_name.unwrap(),
            role_key: req.role_key.unwrap(),
            role_sort: req.role_sort.unwrap(),
            data_scope: req.data_scope,
            menu_check_strictly: req.menu_check_strictly,
            dept_check_strictly: req.dept_check_strictly,
            status: req.status.unwrap_or("0".to_owned()),
            del_flag: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: req.remark,
        };
        let role_active_model = role_model.into_active_model();
        let role = self
            .role_repository
            .create_role(role_active_model, req.menu_ids)
            .await?;
        Ok(role)
    }

    async fn update_role(&self, req: CreateOrUpdateRoleRequest) -> Result<RoleModel> {
        // 检查角色是否存在
        let role_model = self.get_role_by_id(req.role_id.unwrap()).await?;
        if role_model.is_none() {
            return Err(BusinessError(format!("角色不存在")));
        }
        let menu_ids = req.menu_ids;
        let dept_ids = req.dept_ids;
        let mut role_active_model = role_model.unwrap().into_active_model();
        if let Some(role_name) = req.role_name {
            role_active_model.role_name = Set(role_name);
        }
        if let Some(role_key) = req.role_key {
            role_active_model.role_key = Set(role_key);
        }
        if let Some(role_sort) = req.role_sort {
            role_active_model.role_sort = Set(role_sort);
        }
        if let Some(data_scope) = req.data_scope {
            role_active_model.data_scope = Set(Some(data_scope));
        }
        if let Some(menu_check_strictly) = req.menu_check_strictly {
            role_active_model.menu_check_strictly = Set(Some(menu_check_strictly));
        }
        if let Some(dept_check_strictly) = req.dept_check_strictly {
            role_active_model.dept_check_strictly = Set(Some(dept_check_strictly));
        }
        if let Some(status) = req.status {
            role_active_model.status = Set(status);
        }
        if let Some(remark) = req.remark {
            role_active_model.remark = Set(Some(remark));
        }

        let role = self
            .role_repository
            .update_role(role_active_model, menu_ids, dept_ids)
            .await?;
        Ok(role)
    }

    async fn delete_roles(&self, role_ids: Vec<i64>) -> Result<()> {
        // 检查是否包含管理员角色
        if role_ids.contains(&1) {
            return Err(BusinessError(format!("不能删除管理员角色")));
        }

        self.role_repository.delete_role_by_ids(role_ids).await?;
        Ok(())
    }

    async fn check_role_name_unique(&self, role_name: &str, role_id: Option<i64>) -> Result<bool> {
        self.role_repository
            .check_role_name_unique(role_name, role_id)
            .await
    }

    async fn check_role_key_unique(&self, role_key: &str, role_id: Option<i64>) -> Result<bool> {
        self.role_repository
            .check_role_key_unique(role_key, role_id)
            .await
    }

    async fn change_role_status(&self, role_id: i64, status: &str) -> Result<RoleModel> {
        // 检查角色是否存在
        let role = self.get_role_by_id(role_id).await?;
        if role.is_none() {
            return Err(BusinessError(format!("角色不存在")));
        }
        let mut role = role.unwrap().into_active_model();
        // 修改状态
        role.status = Set(status.to_string());

        // 更新角色
        let role = self.role_repository.update_role(role, None, None).await?;
        Ok(role)
    }

    async fn get_role_allocated_users(
        &self,
        req: RoleAuthUserQuery,
    ) -> Result<(Vec<UserInfo>, u64)> {
        let condition = self.build_allocated_users_condition(&req);
        let page_param = PageParam {
            page_size: req.page_size,
            page_num: req.page_num,
            order_by_column: None,
            is_asc: None,
        };
        let (users, total) = self
            .role_repository
            .get_role_allocated_users(req.role_id, condition, &page_param)
            .await?;
        let users = users
            .into_iter()
            .map(|user| UserInfo::from_model(user))
            .collect();
        Ok((users, total))
    }

    async fn get_role_unallocated_users(
        &self,
        req: RoleAuthUserQuery,
    ) -> Result<(Vec<UserInfo>, u64)> {
        let condition = self.build_allocated_users_condition(&req);
        let page_param = PageParam {
            page_size: req.page_size,
            page_num: req.page_num,
            order_by_column: None,
            is_asc: None,
        };
        let (users, total) = self
            .role_repository
            .get_role_unallocated_users(req.role_id, condition, &page_param)
            .await?;
        let users = users
            .into_iter()
            .map(|user| UserInfo::from_model(user))
            .collect();
        Ok((users, total))
    }

    async fn auth_role_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()> {
        // 这里暂时简单实现批量授权
        self.role_repository
            .auth_role_users(role_id, user_ids)
            .await?;
        Ok(())
    }

    async fn cancel_auth_user(&self, role_id: i64, user_id: i64) -> Result<()> {
        self.role_repository
            .cancel_auth_user(role_id, user_id)
            .await?;
        Ok(())
    }

    async fn cancel_auth_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()> {
        self.role_repository
            .cancel_auth_users(role_id, user_ids)
            .await?;
        Ok(())
    }

    async fn get_roles_all(&self) -> Vec<RoleModel> {
        match self.role_repository.get_roles_all().await {
            Ok(roles) => roles,
            Err(e) => {
                error!("[service]get_roles_all: 获取所有角色失败: {}", e);
                vec![]
            }
        }
    }
}
