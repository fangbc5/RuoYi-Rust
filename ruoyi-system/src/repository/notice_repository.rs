// ruoyi-system/src/repository/notice_repository.rs
//! 通知公告仓库实现

use crate::entity::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::{vo::PageParam, Result};
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, Condition, DatabaseConnection, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

/// 通知公告仓库特征
#[async_trait]
pub trait NoticeRepository: Send + Sync {
    /// 获取通知公告列表
    async fn get_notice_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<NoticeModel>, u64)>;

    /// 根据ID获取通知公告
    async fn get_notice_by_id(&self, notice_id: i32) -> Result<Option<NoticeModel>>;

    /// 创建通知公告
    async fn create_notice(&self, notice: NoticeActiveModel) -> Result<NoticeModel>;

    /// 更新通知公告
    async fn update_notice(&self, notice: NoticeActiveModel) -> Result<NoticeModel>;

    /// 删除通知公告
    async fn delete_notices(&self, notice_ids: Vec<i32>) -> Result<u64>;
}

/// 通知公告仓库实现
pub struct NoticeRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<NoticeEntity, NoticeActiveModel>,
}

impl NoticeRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl NoticeRepository for NoticeRepositoryImpl {
    async fn get_notice_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<NoticeModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_asc(NoticeColumn::NoticeId);
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);
        let total = paginator.num_items().await?;
        let notices = paginator.fetch_page(page_param.page_num - 1).await?;
        Ok((notices, total))
    }

    async fn get_notice_by_id(&self, notice_id: i32) -> Result<Option<NoticeModel>> {
        Ok(self.repository.find_by_id(notice_id).await?)
    }

    async fn create_notice(&self, mut notice: NoticeActiveModel) -> Result<NoticeModel> {
        if let Some(user_context) = get_sync_user_context() {
            notice.create_by = Set(Some(user_context.user_name.clone()));
            notice.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        notice.create_time = Set(Some(now));
        notice.update_time = Set(Some(now));
        Ok(notice.insert(self.db.as_ref()).await?)
    }

    async fn update_notice(&self, mut notice: NoticeActiveModel) -> Result<NoticeModel> {
        if let Some(user_context) = get_sync_user_context() {
            notice.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        notice.update_time = Set(Some(now));
        Ok(notice.update(self.db.as_ref()).await?)
    }

    async fn delete_notices(&self, notice_ids: Vec<i32>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(notice_ids).await?)
    }
}
