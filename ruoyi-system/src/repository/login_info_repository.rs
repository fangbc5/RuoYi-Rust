// ruoyi-system/src/repository/login_info_repository.rs
//! 登录日志仓库实现

use crate::entity::prelude::*;
use async_trait::async_trait;
use ruoyi_common::{vo::PageParam, Result};
use ruoyi_framework::db::repository::{BaseRepository, Repository};
use sea_orm::{
    ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};
use std::sync::Arc;

/// 登录日志仓库特征
#[async_trait]
pub trait LoginInfoRepository: Send + Sync {
    /// 获取登录日志列表
    async fn get_login_info_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<LoginInfoModel>, u64)>;

    /// 根据ID获取登录日志
    async fn get_login_info_by_id(&self, info_id: i64) -> Result<Option<LoginInfoModel>>;

    /// 创建登录日志
    async fn create_login_info(&self, login_info: LoginInfoActiveModel) -> Result<LoginInfoModel>;

    /// 更新登录日志
    async fn update_login_info(&self, login_info: LoginInfoActiveModel) -> Result<LoginInfoModel>;

    /// 删除登录日志
    async fn delete_login_infos(&self, info_ids: Vec<i64>) -> Result<u64>;

    /// 清空登录日志
    async fn clean_login_info(&self) -> Result<u64>;
}

/// 登录日志仓库实现
pub struct LoginInfoRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<LoginInfoEntity, LoginInfoActiveModel>,
}

impl LoginInfoRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl LoginInfoRepository for LoginInfoRepositoryImpl {
    async fn get_login_info_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<LoginInfoModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_desc(LoginInfoColumn::LoginTime);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let login_infos = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((login_infos, total))
    }

    async fn get_login_info_by_id(&self, info_id: i64) -> Result<Option<LoginInfoModel>> {
        Ok(self.repository.find_by_id(info_id).await?)
    }

    async fn create_login_info(&self, login_info: LoginInfoActiveModel) -> Result<LoginInfoModel> {
        Ok(login_info.insert(self.db.as_ref()).await?)
    }

    async fn update_login_info(&self, login_info: LoginInfoActiveModel) -> Result<LoginInfoModel> {
        Ok(login_info.update(self.db.as_ref()).await?)
    }

    async fn delete_login_infos(&self, info_ids: Vec<i64>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(info_ids).await?)
    }

    async fn clean_login_info(&self) -> Result<u64> {
        Ok(LoginInfoEntity::delete_many()
            .exec(self.db.as_ref())
            .await
            .map(|res| res.rows_affected)?)
    }
}
