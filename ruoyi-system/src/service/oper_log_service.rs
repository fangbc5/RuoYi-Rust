// ruoyi-system/src/service/oper_log_service.rs
//! 操作日志服务实现

use crate::entity::prelude::*;
use crate::repository::oper_log_repository::OperLogRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ruoyi_common::utils::time::deserialize_optional_datetime;
use ruoyi_common::{vo::PageParam, Result};
use sea_orm::{ColumnTrait, Condition, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 操作日志查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperLogQuery {
    /// 模块标题
    pub title: Option<String>,
    /// 操作地址
    pub oper_ip: Option<String>,
    /// 操作人员
    pub oper_name: Option<String>,
    /// 业务类型（0=其它,1=新增,2=修改,3=删除,4=授权,5=导出,6=导入,7=强退,8=生成代码,9=清空数据）
    pub business_type: Option<i32>,
    /// 操作状态（0正常 1异常）
    pub status: Option<i32>,
    /// 操作开始时间
    #[serde(rename = "params[beginTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub begin_time: Option<DateTime<Utc>>,
    /// 操作结束时间
    #[serde(rename = "params[endTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub end_time: Option<DateTime<Utc>>,
}

/// 创建操作日志请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOperLogRequest {
    /// 模块标题
    pub title: Option<String>,
    /// 业务类型（0=其它,1=新增,2=修改,3=删除,4=授权,5=导出,6=导入,7=强退,8=生成代码,9=清空数据）
    pub business_type: Option<i32>,
    /// 方法名称
    pub method: Option<String>,
    /// 请求方式
    pub request_method: Option<String>,
    /// 操作类别（0=其它,1=后台用户,2=手机端用户）
    pub operator_type: Option<i32>,
    /// 操作人员
    pub oper_name: Option<String>,
    /// 部门名称
    pub dept_name: Option<String>,
    /// 请求URL
    pub oper_url: Option<String>,
    /// 主机地址
    pub oper_ip: Option<String>,
    /// 操作地点
    pub oper_location: Option<String>,
    /// 请求参数
    pub oper_param: Option<String>,
    /// 返回参数
    pub json_result: Option<String>,
    /// 操作状态（0正常 1异常）
    pub status: Option<i32>,
    /// 错误消息
    pub error_msg: Option<String>,
    /// 耗时
    pub cost_time: Option<i64>,
}

/// 操作日志响应数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperLogResponse {
    /// 操作日志信息
    #[serde(flatten)]
    pub oper_log: OperLogModel,
}

/// 操作日志服务特征
#[async_trait]
pub trait OperLogService: Send + Sync {
    /// 获取操作日志列表
    async fn get_oper_log_list(
        &self,
        query: OperLogQuery,
        page_param: PageParam,
    ) -> Result<(Vec<OperLogModel>, u64)>;

    /// 根据ID获取操作日志
    async fn get_oper_log_by_id(&self, oper_id: i64) -> Result<Option<OperLogModel>>;

    /// 记录操作日志
    async fn record_oper_log(&self, req: CreateOperLogRequest) -> Result<OperLogModel>;

    /// 删除操作日志
    async fn delete_oper_logs(&self, oper_ids: Vec<i64>) -> Result<u64>;

    /// 清空操作日志
    async fn clean_oper_log(&self) -> Result<u64>;
}

/// 操作日志服务实现
pub struct OperLogServiceImpl {
    oper_log_repository: Arc<dyn OperLogRepository>,
}

impl OperLogServiceImpl {
    pub fn new(oper_log_repository: Arc<dyn OperLogRepository>) -> Self {
        Self {
            oper_log_repository,
        }
    }

    pub fn build_query_condition(&self, query: &OperLogQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(title) = &query.title {
            condition = condition.add(OperLogColumn::Title.contains(title));
        }

        if let Some(oper_ip) = &query.oper_ip {
            condition = condition.add(OperLogColumn::OperIp.contains(oper_ip));
        }

        if let Some(oper_name) = &query.oper_name {
            condition = condition.add(OperLogColumn::OperName.contains(oper_name));
        }

        if let Some(business_type) = &query.business_type {
            condition = condition.add(OperLogColumn::BusinessType.eq(*business_type));
        }

        if let Some(status) = &query.status {
            condition = condition.add(OperLogColumn::Status.eq(*status));
        }

        // 如果提供了开始时间和结束时间，添加时间范围条件
        if let Some(begin_time) = &query.begin_time {
            condition = condition.add(OperLogColumn::OperTime.gte(*begin_time));
        }

        if let Some(end_time) = &query.end_time {
            condition = condition.add(OperLogColumn::OperTime.lte(*end_time));
        }

        Some(condition)
    }
}

#[async_trait]
impl OperLogService for OperLogServiceImpl {
    async fn get_oper_log_list(
        &self,
        query: OperLogQuery,
        page_param: PageParam,
    ) -> Result<(Vec<OperLogModel>, u64)> {
        self.oper_log_repository
            .get_oper_log_list(self.build_query_condition(&query), page_param)
            .await
    }

    async fn get_oper_log_by_id(&self, oper_id: i64) -> Result<Option<OperLogModel>> {
        self.oper_log_repository.get_oper_log_by_id(oper_id).await
    }

    async fn record_oper_log(&self, req: CreateOperLogRequest) -> Result<OperLogModel> {
        // 验证业务类型
        if let Some(business_type) = req.business_type {
            if !(0..=9).contains(&business_type) {
                return Err(anyhow::anyhow!("业务类型值范围不正确").into());
            }
        }

        // 验证操作类别
        if let Some(operator_type) = req.operator_type {
            if !(0..=2).contains(&operator_type) {
                return Err(anyhow::anyhow!("操作类别值范围不正确").into());
            }
        }

        // 验证操作状态
        if let Some(status) = req.status {
            if !(0..=1).contains(&status) {
                return Err(anyhow::anyhow!("操作状态值范围不正确").into());
            }
        }

        // 创建操作日志记录
        let oper_log = OperLogActiveModel {
            oper_id: Set(0), // 自增ID
            title: Set(req.title),
            business_type: Set(req.business_type),
            method: Set(req.method),
            request_method: Set(req.request_method),
            operator_type: Set(req.operator_type),
            oper_name: Set(req.oper_name),
            dept_name: Set(req.dept_name),
            oper_url: Set(req.oper_url),
            oper_ip: Set(req.oper_ip),
            oper_location: Set(req.oper_location),
            oper_param: Set(req.oper_param),
            json_result: Set(req.json_result),
            status: Set(req.status),
            error_msg: Set(req.error_msg),
            oper_time: Set(Some(Utc::now())),
            cost_time: Set(req.cost_time),
        };

        self.oper_log_repository.create_oper_log(oper_log).await
    }

    async fn delete_oper_logs(&self, oper_ids: Vec<i64>) -> Result<u64> {
        self.oper_log_repository.delete_oper_logs(oper_ids).await
    }

    async fn clean_oper_log(&self) -> Result<u64> {
        self.oper_log_repository.clean_oper_log().await
    }
}
