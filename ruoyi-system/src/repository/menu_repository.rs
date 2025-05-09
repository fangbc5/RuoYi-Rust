// ruoyi-system/src/repository/menu_repository.rs
//! 菜单仓库实现

use crate::entity::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::{vo::PageParam, Result};
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

/// 菜单仓库特征
#[async_trait]
pub trait MenuRepository: Send + Sync {
    /// 根据菜单ID查询菜单
    async fn find_by_id(&self, menu_id: i64) -> Result<Option<MenuModel>>;

    /// 根据菜单名称查询菜单
    async fn find_by_name(&self, menu_name: &str) -> Result<Option<MenuModel>>;

    /// 查询所有菜单
    async fn find_all(&self, condition: Option<Condition>) -> Result<Vec<MenuModel>>;

    /// 查询菜单列表
    async fn find_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<MenuModel>, u64)>;

    /// 根据角色ID查询菜单ID列表
    async fn select_menu_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>>;

    /// 根据用户ID查询菜单列表
    async fn select_menus_by_user_id(&self, user_id: i64) -> Result<Vec<MenuModel>>;

    /// 创建菜单
    async fn create_menu(&self, menu: MenuActiveModel) -> Result<MenuModel>;

    /// 更新菜单
    async fn update_menu(&self, menu: MenuActiveModel) -> Result<MenuModel>;

    /// 删除菜单
    async fn delete_by_id(&self, menu_id: i64) -> Result<()>;

    /// 检查是否存在子菜单
    async fn has_child_by_id(&self, menu_id: i64) -> Result<bool>;

    /// 检查菜单是否被分配
    async fn check_menu_assigned(&self, menu_id: i64) -> Result<bool>;
}

/// 菜单仓库实现
pub struct MenuRepositoryImpl {
    /// 基础仓库
    repository: BaseRepository<MenuEntity, MenuActiveModel>,
    /// 数据库连接
    db: Arc<DatabaseConnection>,
}

impl MenuRepositoryImpl {
    /// 创建菜单仓库
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            repository: BaseRepository::new(db.as_ref().clone()),
            db,
        }
    }
}

#[async_trait]
impl MenuRepository for MenuRepositoryImpl {
    async fn find_by_id(&self, menu_id: i64) -> Result<Option<MenuModel>> {
        Ok(self.repository.find_by_id(menu_id).await?)
    }

    async fn find_by_name(&self, menu_name: &str) -> Result<Option<MenuModel>> {
        Ok(self
            .repository
            .select()
            .filter(MenuColumn::MenuName.eq(menu_name))
            .filter(MenuColumn::Status.eq("0"))
            .one(self.db.as_ref())
            .await?)
    }

    async fn find_all(&self, condition: Option<Condition>) -> Result<Vec<MenuModel>> {
        let mut query = self.repository.select();

        if let Some(cond) = condition {
            query = query.filter(cond);
        }

        Ok(query.all(self.db.as_ref()).await?)
    }

    async fn find_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<MenuModel>, u64)> {
        let mut query = self.repository.select();

        if let Some(cond) = condition {
            query = query.filter(cond);
        }

        // 排序
        query = query
            .order_by(MenuColumn::ParentId, sea_orm::Order::Asc)
            .order_by(MenuColumn::OrderNum, sea_orm::Order::Asc);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let menus = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((menus, total))
    }

    async fn select_menu_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>> {
        let menu_ids = RoleMenuEntity::find()
            .filter(RoleMenuColumn::RoleId.eq(role_id))
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|rm| rm.menu_id)
            .collect();

        Ok(menu_ids)
    }

    async fn select_menus_by_user_id(&self, user_id: i64) -> Result<Vec<MenuModel>> {
        // 用自定义SQL查询用户对应的菜单 (这里简化处理)
        let sql = format!(
            "SELECT m.* FROM sys_user_role ur
        LEFT JOIN sys_role_menu rm ON ur.role_id = rm.role_id
        LEFT JOIN sys_menu m ON rm.menu_id = m.menu_id
        WHERE ur.user_id = {}
        AND m.status = '0'
        ORDER BY m.parent_id ASC, m.order_num ASC",
            user_id
        );
        let menus = MenuEntity::find()
            .from_raw_sql(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::MySql,
                sql,
            ))
            .all(self.db.as_ref())
            .await?;

        Ok(menus)
    }

    async fn create_menu(&self, mut menu: MenuActiveModel) -> Result<MenuModel> {
        if let Some(user_context) = get_sync_user_context() {
            menu.create_by = Set(Some(user_context.user_name.to_string()));
            menu.update_by = Set(Some(user_context.user_name.to_string()));
        }
        let now = Utc::now();
        menu.create_time = Set(Some(now));
        menu.update_time = Set(Some(now));
        let result = menu.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    async fn update_menu(&self, mut menu: MenuActiveModel) -> Result<MenuModel> {
        if let Some(user_context) = get_sync_user_context() {
            menu.update_by = Set(Some(user_context.user_name.to_string()));
        }
        let now = Utc::now();
        menu.update_time = Set(Some(now));
        let result = menu.update(self.db.as_ref()).await?;
        Ok(result)
    }

    async fn delete_by_id(&self, menu_id: i64) -> Result<()> {
        // 删除菜单
        MenuEntity::delete_by_id(menu_id)
            .exec(self.db.as_ref())
            .await?;

        // 删除菜单角色关联
        RoleMenuEntity::delete_many()
            .filter(RoleMenuColumn::MenuId.eq(menu_id))
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    async fn has_child_by_id(&self, menu_id: i64) -> Result<bool> {
        let count = MenuEntity::find()
            .filter(MenuColumn::ParentId.eq(menu_id))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }

    async fn check_menu_assigned(&self, menu_id: i64) -> Result<bool> {
        let count = RoleMenuEntity::find()
            .filter(RoleMenuColumn::MenuId.eq(menu_id))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }
}
