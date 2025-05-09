// ruoyi-system/src/service/dept_service.rs
//! 部门服务

use async_trait::async_trait;
use ruoyi_common::error::Error;
use ruoyi_common::utils::tree::build_tree;
use ruoyi_common::Result;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};
use std::sync::Arc;

use crate::controller::dept_controller::{CreateOrUpdateDeptRequest, DeptQuery};
use crate::entity::prelude::*;
use crate::entity::vo::dept::DeptSelect;
use crate::repository::dept_repository::DeptRepository;

/// 部门服务接口
#[async_trait]
pub trait DeptService: Send + Sync {
    /// 获取部门列表
    async fn get_dept_list(
        &self,
        req: DeptQuery,
        exclude_id: Option<i64>,
    ) -> Result<Vec<DeptModel>>;

    /// 获取部门详情
    async fn get_dept_by_id(&self, dept_id: i64) -> Result<Option<DeptModel>>;

    /// 创建部门
    async fn create_dept(&self, req: CreateOrUpdateDeptRequest) -> Result<DeptModel>;

    /// 更新部门
    async fn update_dept(&self, req: CreateOrUpdateDeptRequest) -> Result<DeptModel>;

    /// 删除部门
    async fn delete_dept(&self, dept_id: i64) -> Result<()>;

    /// 检查部门名称是否唯一
    async fn check_dept_name_unique(
        &self,
        dept_name: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool>;

    /// 获取部门树
    async fn get_dept_tree(&self) -> Result<Vec<Arc<DeptSelect>>>;

    /// 获取部门ID列表
    async fn get_dept_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>>;
}

/// 部门服务实现
pub struct DeptServiceImpl {
    dept_repository: Arc<dyn DeptRepository>,
}

impl DeptServiceImpl {
    /// 创建部门服务
    pub fn new(dept_repository: Arc<dyn DeptRepository>) -> Self {
        Self { dept_repository }
    }

    pub fn build_query(&self, query: DeptQuery, exclude_id: Option<i64>) -> Option<Condition> {
        let mut condition = Condition::all();
        condition = condition.add(DeptColumn::DelFlag.eq("0"));
        if let Some(dept_name) = query.dept_name {
            condition = condition.add(DeptColumn::DeptName.like(dept_name));
        }
        if let Some(status) = query.status {
            condition = condition.add(DeptColumn::Status.eq(status));
        }
        if let Some(exclude_id) = exclude_id {
            condition = condition.add(DeptColumn::DeptId.ne(exclude_id));
        }
        Some(condition)
    }
}

#[async_trait]
impl DeptService for DeptServiceImpl {
    async fn get_dept_list(
        &self,
        query: DeptQuery,
        exclude_id: Option<i64>,
    ) -> Result<Vec<DeptModel>> {
        Ok(self
            .dept_repository
            .find_list(self.build_query(query, exclude_id))
            .await?)
    }

    async fn get_dept_by_id(&self, dept_id: i64) -> Result<Option<DeptModel>> {
        Ok(self.dept_repository.find_by_id(dept_id).await?)
    }

    async fn create_dept(&self, req: CreateOrUpdateDeptRequest) -> Result<DeptModel> {
        let parent_dept = self
            .dept_repository
            .find_by_id(req.parent_id.unwrap())
            .await?;
        if parent_dept.is_none() {
            return Err(Error::BusinessError("父级部门不存在".to_string()));
        }
        let ancestors = if parent_dept.as_ref().unwrap().ancestors.is_some() {
            Some(format!(
                "{},{}",
                parent_dept.as_ref().unwrap().ancestors.as_ref().unwrap(),
                req.parent_id.unwrap()
            ))
        } else {
            None
        };
        let dept_model = DeptModel {
            dept_id: 0,
            dept_name: req.dept_name,
            parent_id: req.parent_id,
            order_num: req.order_num,
            leader: req.leader,
            phone: req.phone,
            email: req.email,
            status: req.status,
            del_flag: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            ancestors,
        };
        let dept_active_model = dept_model.into_active_model();
        Ok(self.dept_repository.create(dept_active_model).await?)
    }

    async fn update_dept(&self, req: CreateOrUpdateDeptRequest) -> Result<DeptModel> {
        let dept_id = req.dept_id.unwrap();
        let dept_model = self.dept_repository.find_by_id(dept_id).await?;
        if dept_model.is_none() {
            return Err(Error::BusinessError("部门不存在".to_string()));
        }
        let mut dept_active_model = dept_model.unwrap().into_active_model();
        if let Some(dept_name) = req.dept_name {
            dept_active_model.dept_name = Set(Some(dept_name));
        }
        if let Some(parent_id) = req.parent_id {
            dept_active_model.parent_id = Set(Some(parent_id));
        }
        if let Some(order_num) = req.order_num {
            dept_active_model.order_num = Set(Some(order_num));
        }
        if let Some(leader) = req.leader {
            dept_active_model.leader = Set(Some(leader));
        }
        if let Some(phone) = req.phone {
            dept_active_model.phone = Set(Some(phone));
        }
        if let Some(email) = req.email {
            dept_active_model.email = Set(Some(email));
        }
        if let Some(status) = req.status {
            dept_active_model.status = Set(Some(status));
        }
        Ok(self.dept_repository.update(dept_active_model).await?)
    }

    async fn delete_dept(&self, dept_id: i64) -> Result<()> {
        // 检查部门是否存在
        let dept_model = self.get_dept_by_id(dept_id).await?;
        if dept_model.is_none() {
            return Err(Error::BusinessError("部门不存在".to_string()));
        }

        // 检查是否存在子部门
        let has_child = self.dept_repository.has_child_by_dept_id(dept_id).await?;
        if has_child {
            return Err(Error::BusinessError("存在子部门，不允许删除".to_string()));
        }

        // 检查是否存在用户
        let has_user = self.dept_repository.has_user_by_dept_id(dept_id).await?;
        if has_user {
            return Err(Error::BusinessError("存在用户，不允许删除".to_string()));
        }

        self.dept_repository
            .delete_by_id(dept_model.unwrap().into_active_model())
            .await?;
        Ok(())
    }

    async fn check_dept_name_unique(
        &self,
        dept_name: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        Ok(self
            .dept_repository
            .check_dept_name_unique(dept_name, parent_id, exclude_id)
            .await?)
    }

    async fn get_dept_tree(&self) -> Result<Vec<Arc<DeptSelect>>> {
        let depts = self.dept_repository.find_all().await?;
        let arc_depts = depts
            .iter()
            .map(|dept| Arc::new(DeptSelect::from_dept_model(dept)))
            .collect();
        let tree = build_tree(arc_depts).await;
        Ok(tree)
    }

    async fn get_dept_ids_by_role_id(&self, role_id: i64) -> Result<Vec<i64>> {
        let dept_ids = self
            .dept_repository
            .get_dept_ids_by_role_id(role_id)
            .await?;
        // 过滤其中是父级部门
        let dept_parnet_ids = self.dept_repository.get_parent_ids().await?;
        let dept_ids = dept_ids
            .iter()
            .filter(|id| !dept_parnet_ids.contains(id))
            .cloned()
            .collect();
        Ok(dept_ids)
    }
}
