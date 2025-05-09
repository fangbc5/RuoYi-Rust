use std::sync::Arc;

use crate::controller::dict_type_controller::{CreateOrUpdateDictTypeRequest, DictTypeQuery};
use crate::entity::prelude::*;
use crate::repository::dict_type_repository::DictTypeRepository;
use async_trait::async_trait;
use ruoyi_common::error::Error;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};

#[async_trait]
pub trait DictTypeService {
    async fn get_dict_type_list(
        &self,
        query: DictTypeQuery,
        page_param: PageParam,
    ) -> Result<(Vec<DictTypeModel>, u64)>;

    async fn get_dict_type(&self, dict_id: i64) -> Result<Option<DictTypeModel>>;

    async fn create_dict_type(
        &self,
        dict_type: CreateOrUpdateDictTypeRequest,
    ) -> Result<DictTypeModel>;

    async fn update_dict_type(
        &self,
        dict_type: CreateOrUpdateDictTypeRequest,
    ) -> Result<DictTypeModel>;

    async fn delete_dict_types(&self, dict_ids: Vec<i64>) -> Result<u64>;

    async fn check_dict_type_unique(&self, dict_type: &str, dict_id: Option<i64>) -> Result<bool>;

    async fn check_dict_type_name_unique(
        &self,
        dict_name: &str,
        dict_id: Option<i64>,
    ) -> Result<bool>;

    async fn get_all_dict_types(&self) -> Result<Vec<DictTypeModel>>;
}

pub struct DictTypeServiceImpl {
    dict_type_repository: Arc<dyn DictTypeRepository>,
}

impl DictTypeServiceImpl {
    pub fn new(dict_type_repository: Arc<dyn DictTypeRepository>) -> Self {
        Self {
            dict_type_repository,
        }
    }
    pub fn build_query_condition(&self, query: &DictTypeQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(dict_name) = &query.dict_name {
            condition = condition.add(DictTypeColumn::DictName.contains(dict_name));
        }
        if let Some(dict_type) = &query.dict_type {
            condition = condition.add(DictTypeColumn::DictType.contains(dict_type));
        }
        if let Some(status) = &query.status {
            condition = condition.add(DictTypeColumn::Status.eq(status));
        }
        if let Some(begin_time) = query.begin_time {
            condition = condition.add(DictTypeColumn::CreateTime.gt(begin_time));
        }
        if let Some(end_time) = query.end_time {
            condition = condition.add(DictTypeColumn::CreateTime.lt(end_time));
        }
        Some(condition)
    }
}

#[async_trait]
impl DictTypeService for DictTypeServiceImpl {
    async fn get_dict_type_list(
        &self,
        query: DictTypeQuery,
        page_param: PageParam,
    ) -> Result<(Vec<DictTypeModel>, u64)> {
        Ok(self
            .dict_type_repository
            .get_dict_type_list(self.build_query_condition(&query), page_param)
            .await?)
    }
    async fn get_dict_type(&self, dict_id: i64) -> Result<Option<DictTypeModel>> {
        Ok(self
            .dict_type_repository
            .get_dict_type_by_id(dict_id)
            .await?)
    }
    async fn create_dict_type(
        &self,
        dict_type: CreateOrUpdateDictTypeRequest,
    ) -> Result<DictTypeModel> {
        let dict_type = DictTypeModel {
            dict_id: 0,
            dict_name: dict_type.dict_name,
            dict_type: dict_type.dict_type,
            status: dict_type.status,
            remark: dict_type.remark,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        };

        Ok(self
            .dict_type_repository
            .create_dict_type(dict_type.into_active_model())
            .await?)
    }
    async fn update_dict_type(&self, req: CreateOrUpdateDictTypeRequest) -> Result<DictTypeModel> {
        let dict_type_id = req.dict_id.unwrap();
        let dict_type_model = self
            .dict_type_repository
            .get_dict_type_by_id(dict_type_id)
            .await?;
        if dict_type_model.is_none() {
            return Err(Error::BusinessError(format!(
                "字典类型不存在: {}",
                dict_type_id
            )));
        }
        let mut dict_type_active_model = dict_type_model.unwrap().into_active_model();
        if let Some(dict_name) = req.dict_name {
            dict_type_active_model.dict_name = Set(Some(dict_name));
        }
        if let Some(dict_type) = req.dict_type {
            dict_type_active_model.dict_type = Set(Some(dict_type));
        }
        if let Some(status) = req.status {
            dict_type_active_model.status = Set(Some(status));
        }
        if let Some(remark) = req.remark {
            dict_type_active_model.remark = Set(Some(remark));
        }
        Ok(self
            .dict_type_repository
            .update_dict_type(dict_type_active_model)
            .await?)
    }
    async fn delete_dict_types(&self, dict_ids: Vec<i64>) -> Result<u64> {
        Ok(self
            .dict_type_repository
            .delete_dict_types(dict_ids)
            .await?)
    }
    async fn check_dict_type_unique(&self, dict_type: &str, dict_id: Option<i64>) -> Result<bool> {
        Ok(self
            .dict_type_repository
            .check_dict_type_unique(dict_type, dict_id)
            .await?)
    }
    async fn check_dict_type_name_unique(
        &self,
        dict_name: &str,
        dict_id: Option<i64>,
    ) -> Result<bool> {
        Ok(self
            .dict_type_repository
            .check_dict_type_name_unique(dict_name, dict_id)
            .await?)
    }
    async fn get_all_dict_types(&self) -> Result<Vec<DictTypeModel>> {
        Ok(self
            .dict_type_repository
            .get_all_dict_types()
            .await?)
    }
}
