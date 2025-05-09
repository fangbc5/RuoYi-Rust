// ruoyi-system/src/repository/dept_repository.rs
//! 部门仓库

use crate::entity::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::Result;
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use std::sync::Arc;

/// 部门仓库特征
#[async_trait]
pub trait DeptRepository: Send + Sync {
    /// 根据ID查询部门
    async fn find_by_id(&self, dept_id: i64) -> Result<Option<DeptModel>>;

    /// 查询部门列表
    async fn find_list(&self, condition: Option<Condition>) -> Result<Vec<DeptModel>>;

    /// 查询所有未删除部门
    async fn find_all(&self) -> Result<Vec<DeptModel>>;

    /// 查询部门名称是否唯一
    async fn check_dept_name_unique(
        &self,
        dept_name: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool>;

    /// 查询部门是否存在子部门
    async fn has_child_by_dept_id(&self, dept_id: i64) -> Result<bool>;

    /// 查询部门是否存在用户
    async fn has_user_by_dept_id(&self, dept_id: i64) -> Result<bool>;

    /// 创建部门
    async fn create(&self, mut dept: DeptActiveModel) -> Result<DeptModel>;

    /// 更新部门
    async fn update(&self, mut dept: DeptActiveModel) -> Result<DeptModel>;

    /// 删除部门
    async fn delete_by_id(&self, mut dept: DeptActiveModel) -> Result<u64>;

    /// 获取部门ID列表
    async fn get_dept_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>>;

    /// 获取所有父级部门ID列表
    async fn get_parent_ids(&self) -> Result<Vec<i64>>;
}

/// 部门仓库实现
pub struct DeptRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<DeptEntity, DeptActiveModel>,
}

impl DeptRepositoryImpl {
    /// 创建部门仓库
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl DeptRepository for DeptRepositoryImpl {
    async fn find_by_id(&self, dept_id: i64) -> Result<Option<DeptModel>> {
        Ok(self.repository.find_by_id(dept_id).await?)
    }

    async fn find_list(&self, condition: Option<Condition>) -> Result<Vec<DeptModel>> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query
            .order_by_asc(DeptColumn::ParentId)
            .order_by_asc(DeptColumn::OrderNum);
        let depts = query.all(self.db.as_ref()).await?;
        Ok(depts)
    }

    async fn find_all(&self) -> Result<Vec<DeptModel>> {
        let depts = self
            .repository
            .select()
            .filter(DeptColumn::DelFlag.eq("0"))
            .all(self.db.as_ref())
            .await?;
        Ok(depts)
    }

    async fn check_dept_name_unique(
        &self,
        dept_name: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(DeptColumn::DelFlag.eq("0"))
            .filter(DeptColumn::DeptName.eq(dept_name))
            .filter(DeptColumn::ParentId.eq(parent_id));

        if let Some(exclude_id) = exclude_id {
            query = query.filter(DeptColumn::DeptId.ne(exclude_id));
        }

        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn has_child_by_dept_id(&self, dept_id: i64) -> Result<bool> {
        let count = DeptEntity::find()
            .filter(DeptColumn::ParentId.eq(dept_id))
            .count(self.db.as_ref())
            .await?;
        Ok(count > 0)
    }

    async fn has_user_by_dept_id(&self, dept_id: i64) -> Result<bool> {
        let count = UserEntity::find()
            .filter(UserColumn::DeptId.eq(dept_id))
            .filter(UserColumn::DelFlag.eq("0"))
            .count(self.db.as_ref())
            .await?;
        Ok(count > 0)
    }

    async fn create(&self, mut dept: DeptActiveModel) -> Result<DeptModel> {
        if let Some(user_context) = get_sync_user_context() {
            dept.create_by = Set(Some(user_context.user_name.to_string()));
            dept.update_by = Set(Some(user_context.user_name.to_string()));
        }

        let now = Utc::now();
        dept.create_time = Set(Some(now));
        dept.update_time = Set(Some(now));
        dept.del_flag = Set(Some("0".to_string()));
        // 插入数据库
        Ok(dept.insert(self.db.as_ref()).await?)
    }

    async fn update(&self, mut dept: DeptActiveModel) -> Result<DeptModel> {
        if let Some(user_context) = get_sync_user_context() {
            dept.update_by = Set(Some(user_context.user_name.to_string()));
        }

        let now = Utc::now();
        dept.update_time = Set(Some(now));
        // 更新数据库
        Ok(dept.update(self.db.as_ref()).await?)
    }

    async fn delete_by_id(&self, mut dept: DeptActiveModel) -> Result<u64> {
        let tx = self.db.begin().await?;
        let dept_id = dept.dept_id.clone().unwrap();
        // 删除部门对应的角色
        RoleDeptEntity::delete_many()
            .filter(RoleDeptColumn::DeptId.eq(dept_id))
            .exec(&tx)
            .await?;
        // 逻辑删除部门
        dept.del_flag = Set(Some("2".to_string()));
        if let Some(user) = get_sync_user_context() {
            dept.update_by = Set(Some(user.user_name.clone()));
        }
        dept.update(&tx).await?;
        tx.commit().await?;
        Ok(1)
    }

    async fn get_dept_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>> {
        let role_depts = RoleDeptEntity::find()
            .filter(RoleDeptColumn::RoleId.eq(role_id))
            .all(self.db.as_ref())
            .await?;
        let dept_ids = role_depts
            .iter()
            .map(|role_dept| role_dept.dept_id)
            .collect();
        Ok(dept_ids)
    }

    async fn get_parent_ids(&self) -> Result<Vec<i64>> {
        let parent_ids = self
            .repository
            .select()
            .select_only()
            .column(DeptColumn::ParentId)
            .into_tuple::<i64>()
            .all(self.db.as_ref())
            .await?;
        Ok(parent_ids)
    }
}
