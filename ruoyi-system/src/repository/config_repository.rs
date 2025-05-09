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
pub trait ConfigRepository: Send + Sync {
    async fn get_config_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<ConfigModel>, u64)>;
    async fn get_all_configs(&self) -> Result<Vec<ConfigModel>>;
    async fn get_config_by_id(&self, config_id: i32) -> Result<Option<ConfigModel>>;
    async fn create_config(&self, mut config: ConfigActiveModel) -> Result<ConfigModel>;
    async fn update_config(&self, mut config: ConfigActiveModel) -> Result<ConfigModel>;
    async fn delete_configs(&self, config_ids: Vec<i32>) -> Result<u64>;
    async fn check_config_name_unique(
        &self,
        config_name: &str,
        config_id: Option<i32>,
    ) -> Result<bool>;
    async fn check_config_key_unique(
        &self,
        config_key: &str,
        config_id: Option<i32>,
    ) -> Result<bool>;
    async fn get_config_by_key(&self, config_key: &str) -> Result<String>;
}

/// 配置仓库实现
pub struct ConfigRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<ConfigEntity, ConfigActiveModel>,
}

impl ConfigRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl ConfigRepository for ConfigRepositoryImpl {
    async fn get_config_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<ConfigModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_asc(ConfigColumn::ConfigId);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let configs = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((configs, total))
    }
    async fn get_all_configs(&self) -> Result<Vec<ConfigModel>> {
        Ok(self.repository.select().all(self.db.as_ref()).await?)
    }
    async fn get_config_by_id(&self, config_id: i32) -> Result<Option<ConfigModel>> {
        Ok(self.repository.find_by_id(config_id).await?)
    }
    async fn create_config(&self, mut config: ConfigActiveModel) -> Result<ConfigModel> {
        if let Some(user_content) = get_sync_user_context() {
            config.create_by = Set(Some(user_content.user_name.clone()));
            config.update_by = Set(Some(user_content.user_name.clone()));
        }
        let now = Utc::now();
        config.create_time = Set(Some(now));
        config.update_time = Set(Some(now));
        Ok(config.insert(self.db.as_ref()).await?)
    }
    async fn update_config(&self, mut config: ConfigActiveModel) -> Result<ConfigModel> {
        if let Some(user_content) = get_sync_user_context() {
            config.update_by = Set(Some(user_content.user_name.clone()));
        }
        let now = Utc::now();
        config.update_time = Set(Some(now));
        Ok(config.update(self.db.as_ref()).await?)
    }
    async fn delete_configs(&self, config_ids: Vec<i32>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(config_ids).await?)
    }
    async fn check_config_name_unique(
        &self,
        config_name: &str,
        config_id: Option<i32>,
    ) -> Result<bool> {
        let mut query = self.repository.select();
        query = query.filter(ConfigColumn::ConfigName.eq(config_name));
        if let Some(config_id) = config_id {
            query = query.filter(ConfigColumn::ConfigId.ne(config_id));
        }
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }
    async fn check_config_key_unique(
        &self,
        config_key: &str,
        config_id: Option<i32>,
    ) -> Result<bool> {
        let mut query = self.repository.select();
        query = query.filter(ConfigColumn::ConfigKey.eq(config_key));
        if let Some(config_id) = config_id {
            query = query.filter(ConfigColumn::ConfigId.ne(config_id));
        }
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }
    async fn get_config_by_key(&self, config_key: &str) -> Result<String> {
        let config = self
            .repository
            .select()
            .filter(ConfigColumn::ConfigKey.eq(config_key))
            .one(self.db.as_ref())
            .await?;
        if let Some(config) = config {
            Ok(config.config_value.unwrap_or_default())
        } else {
            Ok("".to_string())
        }
    }
}
