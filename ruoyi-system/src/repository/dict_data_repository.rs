/// 配置仓库
use crate::entity::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::{vo::PageParam, Result};
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::sync::Arc;

/// 配置仓库特征
#[async_trait]
pub trait DictDataRepository: Send + Sync {
    async fn get_dict_data_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<DictDataModel>, u64)>;
    async fn get_dict_data_by_id(&self, dict_id: i64) -> Result<Option<DictDataModel>>;
    async fn create_dict_data(&self, mut dict_data: DictDataActiveModel) -> Result<DictDataModel>;
    async fn update_dict_data(&self, mut dict_data: DictDataActiveModel) -> Result<DictDataModel>;
    async fn delete_dict_datas(&self, dict_ids: Vec<i64>) -> Result<u64>;
    async fn check_dict_data_label_unique(
        &self,
        dict_type: &str,
        dict_label: &str,
        dict_code: Option<i64>,
    ) -> Result<bool>;
    async fn get_dict_data_by_type(&self, dict_type: &str) -> Result<Vec<DictDataModel>>;
}

/// 配置仓库实现
pub struct DictDataRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<DictDataEntity, DictDataActiveModel>,
}

impl DictDataRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl DictDataRepository for DictDataRepositoryImpl {
    async fn get_dict_data_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<DictDataModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_asc(DictDataColumn::DictSort);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let dict_datas = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((dict_datas, total))
    }
    async fn get_dict_data_by_id(&self, dict_id: i64) -> Result<Option<DictDataModel>> {
        Ok(self.repository.find_by_id(dict_id).await?)
    }
    async fn create_dict_data(&self, mut dict_data: DictDataActiveModel) -> Result<DictDataModel> {
        if let Some(user_context) = get_sync_user_context() {
            dict_data.create_by = Set(Some(user_context.user_name.clone()));
            dict_data.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        dict_data.create_time = Set(Some(now));
        dict_data.update_time = Set(Some(now));
        dict_data.is_default = Set(Some("N".to_string()));
        Ok(dict_data.insert(self.db.as_ref()).await?)
    }
    async fn update_dict_data(&self, mut dict_data: DictDataActiveModel) -> Result<DictDataModel> {
        if let Some(user_context) = get_sync_user_context() {
            dict_data.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        dict_data.update_time = Set(Some(now));
        Ok(dict_data.update(self.db.as_ref()).await?)
    }
    async fn delete_dict_datas(&self, dict_ids: Vec<i64>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(dict_ids).await?)
    }
    async fn check_dict_data_label_unique(
        &self,
        dict_type: &str,
        dict_label: &str,
        dict_code: Option<i64>,
    ) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(DictDataColumn::DictType.eq(dict_type))
            .filter(DictDataColumn::DictLabel.eq(dict_label));
        if let Some(dict_code) = dict_code {
            query = query.filter(DictDataColumn::DictCode.ne(dict_code));
        }
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }
    async fn get_dict_data_by_type(&self, dict_type: &str) -> Result<Vec<DictDataModel>> {
        let dict_data = self
            .repository
            .select()
            .filter(DictDataColumn::DictType.eq(dict_type))
            .order_by_asc(DictDataColumn::DictSort)
            .all(self.db.as_ref())
            .await?;
        Ok(dict_data)
    }
}
