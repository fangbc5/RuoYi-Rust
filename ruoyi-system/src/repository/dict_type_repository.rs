/// 配置仓库
use crate::entity::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
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
pub trait DictTypeRepository: Send + Sync {
    async fn get_dict_type_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<DictTypeModel>, u64)>;

    async fn get_dict_type_by_id(&self, dict_id: i64) -> Result<Option<DictTypeModel>>;

    async fn create_dict_type(&self, mut dict_type: DictTypeActiveModel) -> Result<DictTypeModel>;

    async fn update_dict_type(&self, mut dict_type: DictTypeActiveModel) -> Result<DictTypeModel>;

    async fn delete_dict_types(&self, dict_ids: Vec<i64>) -> Result<u64>;

    async fn check_dict_type_unique(&self, dict_type: &str, dict_id: Option<i64>) -> Result<bool>;

    async fn check_dict_type_name_unique(
        &self,
        dict_name: &str,
        dict_id: Option<i64>,
    ) -> Result<bool>;

    async fn get_all_dict_types(&self) -> Result<Vec<DictTypeModel>>;
}

/// 配置仓库实现
pub struct DictTypeRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<DictTypeEntity, DictTypeActiveModel>,
}

impl DictTypeRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl DictTypeRepository for DictTypeRepositoryImpl {
    async fn get_dict_type_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<DictTypeModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_asc(DictTypeColumn::DictId);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((list, total))
    }

    async fn get_dict_type_by_id(&self, dict_id: i64) -> Result<Option<DictTypeModel>> {
        Ok(self.repository.find_by_id(dict_id).await?)
    }

    async fn create_dict_type(&self, mut dict_type: DictTypeActiveModel) -> Result<DictTypeModel> {
        if let Some(user_context) = get_sync_user_context() {
            dict_type.create_by = Set(Some(user_context.user_name.clone()));
            dict_type.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        dict_type.create_time = Set(Some(now));
        dict_type.update_time = Set(Some(now));
        Ok(dict_type.insert(self.db.as_ref()).await?)
    }

    async fn update_dict_type(&self, mut dict_type: DictTypeActiveModel) -> Result<DictTypeModel> {
        if let Some(user_context) = get_sync_user_context() {
            dict_type.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        dict_type.update_time = Set(Some(now));
        Ok(dict_type.update(self.db.as_ref()).await?)
    }

    async fn delete_dict_types(&self, dict_ids: Vec<i64>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(dict_ids).await?)
    }

    async fn check_dict_type_unique(&self, dict_type: &str, dict_id: Option<i64>) -> Result<bool> {
        let mut query = self.repository.select();
        if let Some(dict_id) = dict_id {
            query = query.filter(DictTypeColumn::DictId.ne(dict_id));
        }
        query = query.filter(DictTypeColumn::DictType.eq(dict_type));
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn check_dict_type_name_unique(
        &self,
        dict_name: &str,
        dict_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = self.repository.select();
        if let Some(dict_id) = dict_id {
            query = query.filter(DictTypeColumn::DictId.ne(dict_id));
        }
        query = query.filter(DictTypeColumn::DictName.eq(dict_name));
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn get_all_dict_types(&self) -> Result<Vec<DictTypeModel>> {
        Ok(self
            .repository
            .select()
            .order_by_asc(DictTypeColumn::DictId)
            .all(self.db.as_ref())
            .await?)
    }
}
