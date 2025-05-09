// ruoyi-system/src/repository/role_repository.rs
//! 角色仓库实现

use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use ruoyi_framework::db::repository::{BaseRepository, Repository};
use ruoyi_framework::web::tls::get_sync_user_context;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use sea_orm::{Condition, QuerySelect};
use std::sync::Arc;

use crate::entity::prelude::*;

// 用户角色联合结构体
pub struct UserWithRole {
    pub user: UserModel,
    pub role_id: Option<i64>,
}

/// 角色仓库特征
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// 根据角色ID查询角色
    async fn find_by_id(&self, role_id: i64) -> Result<Option<RoleModel>>;

    /// 查询角色列表
    async fn find_list(
        &self,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<RoleModel>, u64)>;

    /// 根据用户ID查询角色列表
    async fn get_roles_by_user_id(&self, user_id: i64) -> Result<Vec<RoleModel>>;

    /// 检查角色名称是否已存在
    async fn check_role_name_unique(&self, role_name: &str, role_id: Option<i64>) -> Result<bool>;

    /// 检查角色权限是否已存在
    async fn check_role_key_unique(&self, role_key: &str, role_id: Option<i64>) -> Result<bool>;

    /// 创建角色并分配菜单权限
    async fn create_role(
        &self,
        role: RoleActiveModel,
        menu_ids: Option<Vec<i64>>,
    ) -> Result<RoleModel>;

    /// 更新角色并分配菜单权限
    async fn update_role(
        &self,
        role: RoleActiveModel,
        menu_ids: Option<Vec<i64>>,
        dept_ids: Option<Vec<i64>>,
    ) -> Result<RoleModel>;

    /// 删除角色
    async fn delete_role_by_ids(&self, role_ids: Vec<i64>) -> Result<u64>;

    /// 获取角色分配的用户列表
    async fn get_role_allocated_users(
        &self,
        role_id: i64,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)>;

    /// 获取角色未分配的用户列表
    async fn get_role_unallocated_users(
        &self,
        role_id: i64,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)>;

    /// 分配角色数据权限
    async fn auth_data_scope(&self, role_id: i64, dept_ids: Vec<i64>) -> Result<RoleModel>;

    /// 批量授权用户角色
    async fn auth_role_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()>;

    /// 取消授权用户角色
    async fn cancel_auth_user(&self, role_id: i64, user_id: i64) -> Result<()>;

    /// 批量取消授权用户角色
    async fn cancel_auth_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()>;

    /// 根据角色ID查询用户角色列表
    async fn find_user_role_by_role_id(&self, role_id: i64) -> Result<Vec<i64>>;

    /// 根据角色ID查询用户列表
    async fn find_user_list(&self, role_id: i64) -> Result<Vec<UserWithRole>>;

    /// 获取所有角色
    async fn get_roles_all(&self) -> Result<Vec<RoleModel>>;
}

/// 角色仓库实现
pub struct RoleRepositoryImpl {
    /// 数据库连接
    db: Arc<DatabaseConnection>,
    /// 基础仓库
    repository: BaseRepository<RoleEntity, RoleActiveModel>,
}

impl RoleRepositoryImpl {
    /// 创建角色仓库
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            repository: BaseRepository::new(db.as_ref().clone()),
            db,
        }
    }
}

#[async_trait]
impl RoleRepository for RoleRepositoryImpl {
    async fn find_by_id(&self, role_id: i64) -> Result<Option<RoleModel>> {
        Ok(self.repository.find_by_id(role_id).await?)
    }

    async fn find_list(
        &self,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<RoleModel>, u64)> {
        let mut query = self.repository.select();

        if let Some(cond) = condition {
            query = query.filter(cond);
        } else {
            // 默认只查询未删除的角色
            query = query.filter(RoleColumn::DelFlag.eq("0"));
        }

        query = query.order_by(RoleColumn::RoleSort, sea_orm::Order::Asc);

        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let roles = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((roles, total))
    }

    async fn get_roles_by_user_id(&self, user_id: i64) -> Result<Vec<RoleModel>> {
        // 使用原生SQL查询
        let query = format!(
            "SELECT r.* FROM sys_user_role ur
            LEFT JOIN sys_role r ON r.role_id = ur.role_id 
            WHERE ur.user_id = {} AND r.del_flag = '0'
            ORDER BY r.role_sort ASC",
            user_id
        );

        let roles = RoleEntity::find()
            .from_raw_sql(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::MySql,
                query,
            ))
            .all(self.db.as_ref())
            .await?;

        Ok(roles)
    }

    async fn check_role_name_unique(&self, role_name: &str, role_id: Option<i64>) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(RoleColumn::RoleName.eq(role_name))
            .filter(RoleColumn::DelFlag.eq("0"));
        if let Some(role_id) = role_id {
            query = query.filter(RoleColumn::RoleId.ne(role_id));
        }
        let count = query.count(self.db.as_ref()).await?;

        Ok(count == 0)
    }

    async fn check_role_key_unique(&self, role_key: &str, role_id: Option<i64>) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(RoleColumn::RoleKey.eq(role_key))
            .filter(RoleColumn::DelFlag.eq("0"));
        if let Some(role_id) = role_id {
            query = query.filter(RoleColumn::RoleId.ne(role_id));
        }
        let count = query.count(self.db.as_ref()).await?;

        Ok(count == 0)
    }

    async fn create_role(
        &self,
        mut role: RoleActiveModel,
        menu_ids: Option<Vec<i64>>,
    ) -> Result<RoleModel> {
        // 获取创建人信息
        if let Some(user) = get_sync_user_context() {
            role.create_by = Set(Some(user.user_name.clone()));
            role.update_by = Set(Some(user.user_name.clone()));
        }
        let txn = self.db.begin().await?;

        // 设置创建时间
        let now = Utc::now();
        role.create_time = Set(Some(now));
        role.update_time = Set(Some(now));

        // 默认设置删除标志为0（未删除）
        role.del_flag = Set(Some("0".to_string()));

        // 插入角色
        let role = role.insert(&txn).await?;

        // 添加角色菜单关联
        if let Some(menu_ids) = menu_ids {
            for menu_id in menu_ids {
                let role_menu = RoleMenuActiveModel {
                    role_id: Set(role.role_id),
                    menu_id: Set(menu_id),
                };
                role_menu.insert(&txn).await?;
            }
        }

        txn.commit().await?;
        Ok(role)
    }

    async fn update_role(
        &self,
        mut role: RoleActiveModel,
        menu_ids: Option<Vec<i64>>,
        dept_ids: Option<Vec<i64>>,
    ) -> Result<RoleModel> {
        if let Some(user) = get_sync_user_context() {
            role.update_by = Set(Some(user.user_name.clone()));
        }

        let txn = self.db.begin().await?;
        // 设置更新时间
        role.update_time = Set(Some(Utc::now()));

        // 更新角色
        let role = role.update(&txn).await?;
        if let Some(menu_ids) = menu_ids {
            // 删除旧的菜单关联
            RoleMenuEntity::delete_many()
                .filter(RoleMenuColumn::RoleId.eq(role.role_id))
                .exec(&txn)
                .await?;

            // 添加新的菜单关联
            for menu_id in menu_ids {
                let role_menu = RoleMenuActiveModel {
                    role_id: Set(role.role_id),
                    menu_id: Set(menu_id),
                };
                role_menu.insert(&txn).await?;
            }
        }
        // 添加新的部门关联
        if let Some(dept_ids) = dept_ids {
            // 删除旧的部门关联
            RoleDeptEntity::delete_many()
                .filter(RoleDeptColumn::RoleId.eq(role.role_id))
                .exec(&txn)
                .await?;

            // 添加新的部门关联
            for dept_id in dept_ids {
                let role_dept = RoleDeptActiveModel {
                    role_id: Set(role.role_id),
                    dept_id: Set(dept_id),
                };
                role_dept.insert(&txn).await?;
            }
        }

        txn.commit().await?;
        Ok(role)
    }

    async fn delete_role_by_ids(&self, role_ids: Vec<i64>) -> Result<u64> {
        let txn = self.db.begin().await?;

        // 软删除角色 (设置 del_flag = "2")
        let mut update = RoleActiveModel {
            del_flag: Set(Some("2".to_string())),
            update_time: Set(Some(Utc::now())),
            ..Default::default()
        };
        if let Some(user) = get_sync_user_context() {
            update.update_by = Set(Some(user.user_name.clone()));
        }
        let res = RoleEntity::update_many()
            .filter(RoleColumn::RoleId.is_in(role_ids.clone()))
            .set(update)
            .exec(&txn)
            .await?;

        // 删除角色菜单关联
        RoleMenuEntity::delete_many()
            .filter(RoleMenuColumn::RoleId.is_in(role_ids.clone()))
            .exec(&txn)
            .await?;

        // 删除角色用户关联
        UserRoleEntity::delete_many()
            .filter(UserRoleColumn::RoleId.is_in(role_ids))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(res.rows_affected)
    }

    async fn auth_data_scope(&self, role_id: i64, _dept_ids: Vec<i64>) -> Result<RoleModel> {
        // 查询角色
        let role = self
            .repository
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| DbErr::Custom("角色不存在".to_string()))?;

        // TODO: 实现数据权限相关操作
        // 这里简单返回，实际应该更新角色的数据权限设置并处理部门关联

        Ok(role)
    }

    async fn get_role_allocated_users(
        &self,
        role_id: i64,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)> {
        // 查询出已经授权的用户id
        let user_ids = UserRoleEntity::find()
            .filter(UserRoleColumn::RoleId.eq(role_id))
            .select_only()
            .column(UserRoleColumn::UserId)
            .into_tuple::<i64>()
            .all(self.db.as_ref())
            .await?;
        if user_ids.is_empty() {
            return Ok((vec![], 0));
        }
        // 构建查询
        let mut query = UserEntity::find()
            .filter(UserColumn::UserId.is_in(user_ids))
            .filter(UserColumn::DelFlag.eq("0"));

        // 添加额外的过滤条件
        if let Some(cond) = condition {
            query = query.filter(cond);
        }

        // 排序
        query = query.order_by_asc(UserColumn::UserId);

        // 分页
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let users = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((users, total))
    }

    async fn get_role_unallocated_users(
        &self,
        role_id: i64,
        condition: Option<Condition>,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)> {
        // 查询出userids
        let user_ids = UserRoleEntity::find()
            .filter(UserRoleColumn::RoleId.eq(role_id))
            .select_only()
            .column(UserRoleColumn::UserId)
            .into_tuple::<i64>()
            .all(self.db.as_ref())
            .await?;
        let mut query = UserEntity::find().filter(UserColumn::DelFlag.eq("0"));
        if !user_ids.is_empty() {
            query = query.filter(UserColumn::UserId.is_not_in(user_ids));
        }
        // 添加额外的过滤条件
        if let Some(cond) = condition {
            query = query.filter(cond);
        }

        // 排序
        query = query.order_by_asc(UserColumn::UserId);

        // 分页
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let users = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((users, total))
    }

    async fn cancel_auth_user(&self, role_id: i64, user_id: i64) -> Result<()> {
        // 删除用户角色关联
        UserRoleEntity::delete_many()
            .filter(UserRoleColumn::UserId.eq(user_id))
            .filter(UserRoleColumn::RoleId.eq(role_id))
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    async fn cancel_auth_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()> {
        // 批量删除用户角色关联
        UserRoleEntity::delete_many()
            .filter(UserRoleColumn::UserId.is_in(user_ids))
            .filter(UserRoleColumn::RoleId.eq(role_id))
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    async fn auth_role_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<()> {
        // 开始事务
        let txn = self.db.begin().await?;

        // 批量插入
        for user_id in user_ids {
            let user_role = UserRoleActiveModel {
                user_id: Set(user_id),
                role_id: Set(role_id),
            };
            user_role.insert(&txn).await?;
        }

        // 提交事务
        txn.commit().await?;
        Ok(())
    }

    async fn find_user_role_by_role_id(&self, role_id: i64) -> Result<Vec<i64>> {
        // 直接使用表连接查询用户ID
        let user_ids = UserRoleEntity::find()
            .filter(UserRoleColumn::RoleId.eq(role_id))
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|ur| ur.user_id)
            .collect();

        Ok(user_ids)
    }

    async fn find_user_list(&self, role_id: i64) -> Result<Vec<UserWithRole>> {
        // 直接使用原生查询
        let query = format!(
            "SELECT u.* FROM sys_user u 
            JOIN sys_user_role ur ON u.user_id = ur.user_id 
            WHERE ur.role_id = {} AND u.del_flag = '0'",
            role_id
        );

        let users = UserEntity::find()
            .from_raw_sql(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::MySql,
                query,
            ))
            .all(self.db.as_ref())
            .await?;

        let mut user_with_roles = Vec::new();
        for user in users {
            user_with_roles.push(UserWithRole {
                user,
                role_id: Some(role_id),
            });
        }

        Ok(user_with_roles)
    }

    async fn get_roles_all(&self) -> Result<Vec<RoleModel>> {
        Ok(self.repository.find_all().await?)
    }
}
