use std::sync::Arc;

use async_trait::async_trait;
use log::info;
use ruoyi_common::{constants, Result};
use ruoyi_common::{error::Error, vo::PageParam};
use ruoyi_framework::cache::get_global_cache;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};

use crate::{
    controller::dict_data_controller::{CreateOrUpdateDictDataRequest, DictDataQuery},
    entity::prelude::*,
    repository::dict_data_repository::DictDataRepository,
};

#[async_trait]
pub trait DictDataService {
    async fn get_dict_data_list(
        &self,
        query: DictDataQuery,
        page_param: PageParam,
    ) -> Result<(Vec<DictDataModel>, u64)>;
    async fn get_dict_data_by_id(&self, dict_id: i64) -> Result<Option<DictDataModel>>;
    async fn create_dict_data(&self, req: CreateOrUpdateDictDataRequest) -> Result<DictDataModel>;
    async fn update_dict_data(&self, req: CreateOrUpdateDictDataRequest) -> Result<DictDataModel>;
    async fn delete_dict_datas(&self, dict_ids: Vec<i64>) -> Result<u64>;
    async fn check_dict_data_label_unique(
        &self,
        dict_type: &str,
        dict_label: &str,
        dict_code: Option<i64>,
    ) -> Result<bool>;
    async fn get_dict_data_by_type(&self, dict_type: &str) -> Result<Vec<DictDataModel>>;
}

pub struct DictDataServiceImpl {
    dict_data_repository: Arc<dyn DictDataRepository>,
}

impl DictDataServiceImpl {
    pub fn new(dict_data_repository: Arc<dyn DictDataRepository>) -> Self {
        Self {
            dict_data_repository,
        }
    }
    pub fn build_query_condition(&self, query: &DictDataQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(dict_type) = &query.dict_type {
            condition = condition.add(DictDataColumn::DictType.eq(dict_type));
        }
        if let Some(dict_label) = &query.dict_label {
            condition = condition.add(DictDataColumn::DictLabel.contains(dict_label));
        }
        if let Some(status) = &query.status {
            condition = condition.add(DictDataColumn::Status.eq(status));
        }
        Some(condition)
    }
}

#[async_trait]
impl DictDataService for DictDataServiceImpl {
    async fn get_dict_data_list(
        &self,
        query: DictDataQuery,
        page_param: PageParam,
    ) -> Result<(Vec<DictDataModel>, u64)> {
        Ok(self
            .dict_data_repository
            .get_dict_data_list(self.build_query_condition(&query), page_param)
            .await?)
    }
    async fn get_dict_data_by_id(&self, dict_id: i64) -> Result<Option<DictDataModel>> {
        Ok(self
            .dict_data_repository
            .get_dict_data_by_id(dict_id)
            .await?)
    }
    async fn create_dict_data(&self, req: CreateOrUpdateDictDataRequest) -> Result<DictDataModel> {
        let dict_data = DictDataModel {
            dict_code: 0,
            dict_label: req.dict_label,
            dict_value: req.dict_value,
            dict_type: req.dict_type,
            status: req.status,
            css_class: req.css_class,
            list_class: req.list_class,
            is_default: req.is_default,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: req.remark,
            dict_sort: req.dict_sort,
        };
        Ok(self
            .dict_data_repository
            .create_dict_data(dict_data.into_active_model())
            .await?)
    }
    async fn update_dict_data(&self, req: CreateOrUpdateDictDataRequest) -> Result<DictDataModel> {
        let dict_data_id = req.dict_code.unwrap();
        let dict_data_model = self
            .dict_data_repository
            .get_dict_data_by_id(dict_data_id)
            .await?;
        if dict_data_model.is_none() {
            return Err(Error::BusinessError(format!(
                "字典数据不存在: {}",
                dict_data_id
            )));
        }
        let mut dict_data_active_model = dict_data_model.unwrap().into_active_model();
        if let Some(dict_label) = req.dict_label {
            dict_data_active_model.dict_label = Set(Some(dict_label));
        }
        if let Some(dict_value) = req.dict_value {
            dict_data_active_model.dict_value = Set(Some(dict_value));
        }
        if let Some(status) = req.status {
            dict_data_active_model.status = Set(Some(status));
        }
        if let Some(css_class) = req.css_class {
            dict_data_active_model.css_class = Set(Some(css_class));
        }
        if let Some(list_class) = req.list_class {
            dict_data_active_model.list_class = Set(Some(list_class));
        }
        if let Some(is_default) = req.is_default {
            dict_data_active_model.is_default = Set(Some(is_default));
        }
        if let Some(remark) = req.remark {
            dict_data_active_model.remark = Set(Some(remark));
        }
        if let Some(dict_sort) = req.dict_sort {
            dict_data_active_model.dict_sort = Set(Some(dict_sort));
        }
        Ok(self
            .dict_data_repository
            .update_dict_data(dict_data_active_model)
            .await?)
    }
    async fn delete_dict_datas(&self, dict_ids: Vec<i64>) -> Result<u64> {
        Ok(self
            .dict_data_repository
            .delete_dict_datas(dict_ids)
            .await?)
    }
    async fn check_dict_data_label_unique(
        &self,
        dict_type: &str,
        dict_label: &str,
        dict_code: Option<i64>,
    ) -> Result<bool> {
        Ok(self
            .dict_data_repository
            .check_dict_data_label_unique(dict_type, dict_label, dict_code)
            .await?)
    }
    async fn get_dict_data_by_type(&self, dict_type: &str) -> Result<Vec<DictDataModel>> {
        // 从缓存中获取字典数据
        if let Ok(cache) = get_global_cache() {
            if let Ok(dict_data) = cache
                .get_string(&format!(
                    "{}{}",
                    constants::cache::SYS_DICT_PREFIX,
                    &dict_type
                ))
                .await
            {
                if let Some(dict_data) = dict_data {
                    if let Ok(dict_data) = serde_json::from_str::<Vec<DictDataModel>>(&dict_data) {
                        info!("缓存命中获取字典数据: {:?}", dict_data);
                        return Ok(dict_data);
                    }
                }
            }
        }
        info!("缓存未命中获取字典数据: {}", dict_type);
        let dict_data = self
            .dict_data_repository
            .get_dict_data_by_type(dict_type)
            .await?;
        Ok(dict_data)
    }
}
