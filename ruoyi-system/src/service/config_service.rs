use crate::{controller::config_controller::{ConfigQuery, CreateOrUpdateConfigRequest}, repository::config_repository::ConfigRepository};
use ruoyi_common::{error::Error, Result};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};
use std::sync::Arc;
use ruoyi_common::vo::PageParam;
use crate::entity::prelude::*;

#[async_trait]
pub trait ConfigService: Send + Sync {
    async fn get_config_list(&self, query: ConfigQuery, page_param: PageParam) -> Result<(Vec<ConfigModel>, u64)>;
    async fn get_all_configs(&self) -> Result<Vec<ConfigModel>>;
    async fn get_config_by_id(&self, config_id: i32) -> Result<Option<ConfigModel>>;
    async fn create_config(&self, req: CreateOrUpdateConfigRequest) -> Result<ConfigModel>;
    async fn update_config(&self, req: CreateOrUpdateConfigRequest) -> Result<ConfigModel>;
    async fn delete_configs(&self, config_ids: Vec<i32>) -> Result<u64>;
    async fn check_config_name_unique(&self, config_name: &str, config_id: Option<i32>) -> Result<bool>;
    async fn check_config_key_unique(&self, config_key: &str, config_id: Option<i32>) -> Result<bool>;
    async fn get_config_by_key(&self, config_key: &str) -> Result<String>;
}

pub struct ConfigServiceImpl {
    config_repository: Arc<dyn ConfigRepository>,
}

impl ConfigServiceImpl {
    pub fn new(config_repository: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repository }
    }
    pub fn build_query_condition(&self, query: &ConfigQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(config_key) = &query.config_key {
            condition = condition.add(ConfigColumn::ConfigKey.contains(config_key));
        }
        if let Some(config_name) = &query.config_name {
            condition = condition.add(ConfigColumn::ConfigName.contains(config_name));
        }
        if let Some(config_type) = &query.config_type {
            condition = condition.add(ConfigColumn::ConfigType.eq(config_type));
        }
        if let Some(begin_time) = query.begin_time {
            condition = condition.add(ConfigColumn::CreateTime.gte(begin_time));
        }
        if let Some(end_time) = query.end_time {
            condition = condition.add(ConfigColumn::CreateTime.lte(end_time));
        }
        Some(condition)
    }
}

#[async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config_list(&self, query: ConfigQuery, page_param: PageParam) -> Result<(Vec<ConfigModel>, u64)> {
        Ok(self.config_repository.get_config_list(self.build_query_condition(&query), page_param).await?)
    }
    async fn get_all_configs(&self) -> Result<Vec<ConfigModel>> {
        Ok(self.config_repository.get_all_configs().await?)
    }
    async fn get_config_by_id(&self, config_id: i32) -> Result<Option<ConfigModel>> {
        Ok(self.config_repository.get_config_by_id(config_id).await?)
    }
    async fn create_config(&self, req: CreateOrUpdateConfigRequest) -> Result<ConfigModel> {
        let model = ConfigModel {
            config_id: 0,
            config_key: req.config_key,
            config_name: req.config_name,
            config_value: req.config_value,
            config_type: req.config_type,
            remark: req.remark,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        };
        Ok(self.config_repository.create_config(model.into_active_model()).await?)
    }
    async fn update_config(&self, req: CreateOrUpdateConfigRequest) -> Result<ConfigModel> {
        let config_id = req.config_id.unwrap();
        let model = self.config_repository.get_config_by_id(config_id).await?;
        if model.is_none() {
            return Err(Error::BusinessError(format!("配置不存在: {}", config_id)));
        }
        let mut active_model = model.unwrap().into_active_model();
        if let Some(config_key) = req.config_key {
            active_model.config_key = Set(Some(config_key));
        }
        if let Some(config_name) = req.config_name {
            active_model.config_name = Set(Some(config_name));
        }
        if let Some(config_value) = req.config_value {
            active_model.config_value = Set(Some(config_value));
        }
        if let Some(config_type) = req.config_type {
            active_model.config_type = Set(Some(config_type));
        }
        if let Some(remark) = req.remark {
            active_model.remark = Set(Some(remark));
        }
        Ok(self.config_repository.update_config(active_model).await?)
    }
    async fn delete_configs(&self, config_ids: Vec<i32>) -> Result<u64> {
        Ok(self.config_repository.delete_configs(config_ids).await?)
    }
    async fn check_config_name_unique(&self, config_name: &str, config_id: Option<i32>) -> Result<bool> {
        Ok(self.config_repository.check_config_name_unique(config_name, config_id).await?)
    }
    async fn check_config_key_unique(&self, config_key: &str, config_id: Option<i32>) -> Result<bool> {
        Ok(self.config_repository.check_config_key_unique(config_key, config_id).await?)
    }
    async fn get_config_by_key(&self, config_key: &str) -> Result<String> {
        Ok(self.config_repository.get_config_by_key(config_key).await?)
    }
}
