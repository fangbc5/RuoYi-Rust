// ruoyi-system/src/repository/oper_log_repository.rs
//! 操作日志仓库实现

use crate::entity::prelude::*;
use async_trait::async_trait;
use ruoyi_common::{utils, vo::PageParam, Result};
use ruoyi_framework::db::repository::{BaseRepository, Repository};
use sea_orm::{
    ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use std::{str::FromStr, sync::Arc};

/// 操作日志仓库特征
#[async_trait]
pub trait OperLogRepository: Send + Sync {
    /// 获取操作日志列表
    async fn get_oper_log_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<OperLogModel>, u64)>;

    /// 根据ID获取操作日志
    async fn get_oper_log_by_id(&self, oper_id: i64) -> Result<Option<OperLogModel>>;

    /// 创建操作日志
    async fn create_oper_log(&self, oper_log: OperLogActiveModel) -> Result<OperLogModel>;

    /// 更新操作日志
    async fn update_oper_log(&self, oper_log: OperLogActiveModel) -> Result<OperLogModel>;

    /// 删除操作日志
    async fn delete_oper_logs(&self, oper_ids: Vec<i64>) -> Result<u64>;

    /// 清空操作日志
    async fn clean_oper_log(&self) -> Result<u64>;
}

/// 操作日志仓库实现
pub struct OperLogRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<OperLogEntity, OperLogActiveModel>,
}

impl OperLogRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl OperLogRepository for OperLogRepositoryImpl {
    async fn get_oper_log_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<OperLogModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        if let Some(order_by_column) = page_param.order_by_column {
            let order_by_column = utils::string::to_snake_case(&order_by_column);
            if page_param.is_asc == Some("ascending".to_string()) {
                query = query.order_by_asc(OperLogColumn::from_str(&order_by_column).unwrap());
            } else {
                query = query.order_by_desc(OperLogColumn::from_str(&order_by_column).unwrap());
            }
        }
        query = query.order_by_desc(OperLogColumn::OperTime);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let oper_logs = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((oper_logs, total))
    }

    async fn get_oper_log_by_id(&self, oper_id: i64) -> Result<Option<OperLogModel>> {
        Ok(self.repository.find_by_id(oper_id).await?)
    }

    async fn create_oper_log(&self, oper_log: OperLogActiveModel) -> Result<OperLogModel> {
        Ok(oper_log.insert(self.db.as_ref()).await?)
    }

    async fn update_oper_log(&self, oper_log: OperLogActiveModel) -> Result<OperLogModel> {
        Ok(oper_log.update(self.db.as_ref()).await?)
    }

    async fn delete_oper_logs(&self, oper_ids: Vec<i64>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(oper_ids).await?)
    }

    async fn clean_oper_log(&self) -> Result<u64> {
        Ok(OperLogEntity::delete_many()
            .exec(self.db.as_ref())
            .await
            .map(|res| res.rows_affected)?)
    }
}
