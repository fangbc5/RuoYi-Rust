// ruoyi-system/src/service/login_info_service.rs
//! 登录日志服务实现

use crate::entity::prelude::*;
use crate::repository::login_info_repository::LoginInfoRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ruoyi_common::utils::time::deserialize_optional_datetime;
use ruoyi_common::{vo::PageParam, Result};
use sea_orm::{ColumnTrait, Condition, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 登录日志查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfoQuery {
    /// 用户名
    pub user_name: Option<String>,
    /// IP地址
    pub ipaddr: Option<String>,
    /// 登录状态（0成功 1失败）
    pub status: Option<String>,
    /// 登录开始时间
    #[serde(rename = "params[beginTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub begin_time: Option<DateTime<Utc>>,
    /// 登录结束时间
    #[serde(rename = "params[endTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub end_time: Option<DateTime<Utc>>,
}

/// 创建登录日志请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoginInfoRequest {
    /// 用户名
    pub user_name: Option<String>,
    /// IP地址
    pub ipaddr: Option<String>,
    /// 登录地点
    pub login_location: Option<String>,
    /// 浏览器类型
    pub browser: Option<String>,
    /// 操作系统
    pub os: Option<String>,
    /// 登录状态（0成功 1失败）
    pub status: Option<String>,
    /// 提示消息
    pub msg: Option<String>,
}

/// 登录日志返回数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfoResponse {
    /// 登录信息
    #[serde(flatten)]
    pub login_info: LoginInfoModel,
}

/// 登录日志服务特征
#[async_trait]
pub trait LoginInfoService: Send + Sync {
    /// 获取登录日志列表
    async fn get_login_info_list(
        &self,
        query: LoginInfoQuery,
        page_param: PageParam,
    ) -> Result<(Vec<LoginInfoModel>, u64)>;

    /// 根据ID获取登录日志
    async fn get_login_info_by_id(&self, info_id: i64) -> Result<Option<LoginInfoModel>>;

    /// 记录登录信息
    async fn record_login_info(&self, req: CreateLoginInfoRequest) -> Result<LoginInfoModel>;

    /// 删除登录日志
    async fn delete_login_infos(&self, info_ids: Vec<i64>) -> Result<u64>;

    /// 清空登录日志
    async fn clean_login_info(&self) -> Result<u64>;
}

/// 登录日志服务实现
pub struct LoginInfoServiceImpl {
    login_info_repository: Arc<dyn LoginInfoRepository>,
}

impl LoginInfoServiceImpl {
    pub fn new(login_info_repository: Arc<dyn LoginInfoRepository>) -> Self {
        Self {
            login_info_repository,
        }
    }

    pub fn build_query_condition(&self, query: &LoginInfoQuery) -> Option<Condition> {
        let mut condition = Condition::all();

        if let Some(user_name) = &query.user_name {
            condition = condition.add(LoginInfoColumn::UserName.contains(user_name));
        }

        if let Some(ipaddr) = &query.ipaddr {
            condition = condition.add(LoginInfoColumn::Ipaddr.contains(ipaddr));
        }

        if let Some(status) = &query.status {
            condition = condition.add(LoginInfoColumn::Status.eq(status));
        }

        // 如果提供了开始时间和结束时间，添加时间范围条件
        if let Some(begin_time) = &query.begin_time {
            condition = condition.add(LoginInfoColumn::LoginTime.gte(*begin_time));
        }

        if let Some(end_time) = &query.end_time {
            condition = condition.add(LoginInfoColumn::LoginTime.lte(*end_time));
        }

        Some(condition)
    }
}

#[async_trait]
impl LoginInfoService for LoginInfoServiceImpl {
    async fn get_login_info_list(
        &self,
        query: LoginInfoQuery,
        page_param: PageParam,
    ) -> Result<(Vec<LoginInfoModel>, u64)> {
        self.login_info_repository
            .get_login_info_list(self.build_query_condition(&query), page_param)
            .await
    }

    async fn get_login_info_by_id(&self, info_id: i64) -> Result<Option<LoginInfoModel>> {
        self.login_info_repository
            .get_login_info_by_id(info_id)
            .await
    }

    async fn record_login_info(&self, req: CreateLoginInfoRequest) -> Result<LoginInfoModel> {
        // 创建登录信息记录
        let login_info = LoginInfoActiveModel {
            info_id: Set(0), // 自增ID
            user_name: Set(req.user_name),
            ipaddr: Set(req.ipaddr),
            login_location: Set(req.login_location),
            browser: Set(req.browser),
            os: Set(req.os),
            status: Set(req.status),
            msg: Set(req.msg),
            login_time: Set(Some(Utc::now())),
        };

        self.login_info_repository
            .create_login_info(login_info)
            .await
    }

    async fn delete_login_infos(&self, info_ids: Vec<i64>) -> Result<u64> {
        self.login_info_repository
            .delete_login_infos(info_ids)
            .await
    }

    async fn clean_login_info(&self) -> Result<u64> {
        self.login_info_repository.clean_login_info().await
    }
}
