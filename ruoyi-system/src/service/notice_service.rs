// ruoyi-system/src/service/notice_service.rs
//! 通知公告服务实现

use crate::{
    entity::notice::{ActiveModel as NoticeActiveModel, Model as NoticeModel},
    repository::notice_repository::NoticeRepository,
};
use async_trait::async_trait;
use ruoyi_common::{error::Error, utils::string::string_to_vec_u8, vo::PageParam, Result};
use sea_orm::{ColumnTrait, Condition, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 通知公告查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeQuery {
    /// 公告标题
    pub notice_title: Option<String>,
    /// 公告类型（1通知 2公告）
    pub notice_type: Option<String>,
    /// 创建者
    pub create_by: Option<String>,
    /// 公告状态（0正常 1关闭）
    pub status: Option<String>,
}

/// 创建或更新通知公告请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateNoticeRequest {
    /// 公告ID
    pub notice_id: Option<i32>,
    /// 公告标题
    pub notice_title: Option<String>,
    /// 公告类型（1通知 2公告）
    pub notice_type: Option<String>,
    /// 公告内容
    pub notice_content: Option<String>,
    /// 公告状态（0正常 1关闭）
    pub status: Option<String>,
    /// 备注
    pub remark: Option<String>,
}

/// 通知公告返回数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeResponse {
    /// 公告信息
    #[serde(flatten)]
    pub notice: NoticeModel,
}

/// 通知公告服务特征
#[async_trait]
pub trait NoticeService: Send + Sync {
    /// 获取通知公告列表
    async fn get_notice_list(
        &self,
        query: NoticeQuery,
        page_param: PageParam,
    ) -> Result<(Vec<NoticeModel>, u64)>;

    /// 根据ID获取通知公告
    async fn get_notice_by_id(&self, notice_id: i32) -> Result<Option<NoticeModel>>;

    /// 创建通知公告
    async fn create_notice(&self, req: CreateOrUpdateNoticeRequest) -> Result<NoticeModel>;

    /// 更新通知公告
    async fn update_notice(&self, req: CreateOrUpdateNoticeRequest) -> Result<NoticeModel>;

    /// 删除通知公告
    async fn delete_notices(&self, notice_ids: Vec<i32>) -> Result<u64>;
}

/// 通知公告服务实现
pub struct NoticeServiceImpl {
    notice_repository: Arc<dyn NoticeRepository>,
}

impl NoticeServiceImpl {
    pub fn new(notice_repository: Arc<dyn NoticeRepository>) -> Self {
        Self { notice_repository }
    }

    pub fn build_query_condition(&self, query: &NoticeQuery) -> Option<Condition> {
        let mut condition = Condition::all();

        if let Some(notice_title) = &query.notice_title {
            condition =
                condition.add(crate::entity::notice::Column::NoticeTitle.contains(notice_title));
        }

        if let Some(notice_type) = &query.notice_type {
            condition = condition.add(crate::entity::notice::Column::NoticeType.eq(notice_type));
        }

        if let Some(create_by) = &query.create_by {
            condition = condition.add(crate::entity::notice::Column::CreateBy.eq(create_by));
        }

        if let Some(status) = &query.status {
            condition = condition.add(crate::entity::notice::Column::Status.eq(status));
        }

        Some(condition)
    }
}

#[async_trait]
impl NoticeService for NoticeServiceImpl {
    async fn get_notice_list(
        &self,
        query: NoticeQuery,
        page_param: PageParam,
    ) -> Result<(Vec<NoticeModel>, u64)> {
        self.notice_repository
            .get_notice_list(self.build_query_condition(&query), page_param)
            .await
    }

    async fn get_notice_by_id(&self, notice_id: i32) -> Result<Option<NoticeModel>> {
        self.notice_repository.get_notice_by_id(notice_id).await
    }

    async fn create_notice(&self, req: CreateOrUpdateNoticeRequest) -> Result<NoticeModel> {
        let notice = NoticeActiveModel {
            notice_id: Set(0), // 自增ID
            notice_title: Set(req.notice_title.unwrap_or_default()),
            notice_type: Set(req.notice_type.unwrap_or_default()),
            notice_content: Set(Some(string_to_vec_u8(
                &req.notice_content.unwrap_or_default(),
            ))),
            status: Set(req.status),
            remark: Set(req.remark),
            ..Default::default()
        };

        self.notice_repository.create_notice(notice).await
    }

    async fn update_notice(&self, req: CreateOrUpdateNoticeRequest) -> Result<NoticeModel> {
        let notice_id = req.notice_id.unwrap_or_default();
        let notice_opt = self.notice_repository.get_notice_by_id(notice_id).await?;

        if let Some(old_notice) = notice_opt {
            // 将旧的通知转换为 ActiveModel
            let mut notice: NoticeActiveModel = old_notice.into();

            // 更新字段
            if let Some(notice_title) = req.notice_title {
                notice.notice_title = Set(notice_title);
            }

            if let Some(notice_type) = req.notice_type {
                notice.notice_type = Set(notice_type);
            }

            if let Some(notice_content) = req.notice_content {
                notice.notice_content = Set(Some(string_to_vec_u8(&notice_content)));
            }

            if let Some(status) = req.status {
                notice.status = Set(Some(status));
            }
            if let Some(remark) = req.remark {
                notice.remark = Set(Some(remark));
            }

            self.notice_repository.update_notice(notice).await
        } else {
            Err(Error::NotFound(format!("通知公告不存在: {}", notice_id)))
        }
    }

    async fn delete_notices(&self, notice_ids: Vec<i32>) -> Result<u64> {
        self.notice_repository.delete_notices(notice_ids).await
    }
}
