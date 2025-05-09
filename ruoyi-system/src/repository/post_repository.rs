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
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

/// 岗位仓库特征
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn get_post_by_id(&self, id: i64) -> Result<Option<PostModel>>;
    async fn get_post_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<PostModel>, u64)>;
    async fn create_post(&self, mut post: PostActiveModel) -> Result<PostModel>;
    async fn update_post(&self, mut post: PostActiveModel) -> Result<PostModel>;
    async fn delete_posts(&self, ids: Vec<i64>) -> Result<u64>;
    async fn check_post_name_unique(
        &self,
        post_name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool>;
    async fn check_post_code_unique(
        &self,
        post_code: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool>;
    async fn get_posts_by_user_id(&self, user_id: i64) -> Result<Vec<PostModel>>;
    async fn get_posts_all(&self) -> Result<Vec<PostModel>>;
    async fn get_post_ids_by_user_id(&self, user_id: i64) -> Result<Vec<i64>>;
}

/// 岗位仓库实现
pub struct PostRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<PostEntity, PostActiveModel>,
}

impl PostRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new(db.as_ref().clone()),
        }
    }
}

#[async_trait]
impl PostRepository for PostRepositoryImpl {
    async fn get_post_by_id(&self, id: i64) -> Result<Option<PostModel>> {
        let post = self.repository.find_by_id(id).await?;
        Ok(post)
    }

    async fn get_post_list(
        &self,
        condition: Option<Condition>,
        page_param: PageParam,
    ) -> Result<(Vec<PostModel>, u64)> {
        let mut query = self.repository.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        query = query.order_by_asc(PostColumn::PostSort);

        let page = query.paginate(self.db.as_ref(), page_param.page_size);
        let totals = page.num_items().await?;
        let posts = page.fetch_page(page_param.page_num - 1).await?;
        Ok((posts, totals))
    }

    async fn create_post(&self, mut post: PostActiveModel) -> Result<PostModel> {
        if let Some(user_context) = get_sync_user_context() {
            post.create_by = Set(Some(user_context.user_name.clone()));
            post.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        post.create_time = Set(Some(now));
        post.update_time = Set(Some(now));
        post.status = Set("0".to_owned());
        Ok(post.insert(self.db.as_ref()).await?)
    }

    async fn update_post(&self, mut post: PostActiveModel) -> Result<PostModel> {
        if let Some(user_context) = get_sync_user_context() {
            post.update_by = Set(Some(user_context.user_name.clone()));
        }
        let now = Utc::now();
        post.update_time = Set(Some(now));
        Ok(post.update(self.db.as_ref()).await?)
    }

    async fn delete_posts(&self, ids: Vec<i64>) -> Result<u64> {
        Ok(self.repository.delete_by_ids(ids).await?)
    }

    async fn check_post_name_unique(
        &self,
        post_name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(PostColumn::PostName.eq(post_name));
        if let Some(exclude_id) = exclude_id {
            query = query.filter(PostColumn::PostId.ne(exclude_id));
        }
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn check_post_code_unique(
        &self,
        post_code: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = self
            .repository
            .select()
            .filter(PostColumn::PostCode.eq(post_code));
        if let Some(exclude_id) = exclude_id {
            query = query.filter(PostColumn::PostId.ne(exclude_id));
        }
        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn get_post_ids_by_user_id(&self, user_id: i64) -> Result<Vec<i64>> {
        let post_ids = UserPostEntity::find()
            .filter(UserPostColumn::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await?;
        let post_ids = post_ids.iter().map(|post| post.post_id).collect();
        Ok(post_ids)
    }

    async fn get_posts_by_user_id(&self, user_id: i64) -> Result<Vec<PostModel>> {
        let sql = format!(
            "SELECT p.* FROM sys_user_post up
            LEFT JOIN sys_post p ON up.post_id = p.post_id
            WHERE up.user_id = {} and p.status = '0'",
            user_id
        );
        let posts = self
            .repository
            .select()
            .from_raw_sql(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::MySql,
                sql,
            ))
            .all(self.db.as_ref())
            .await?;
        Ok(posts)
    }

    async fn get_posts_all(&self) -> Result<Vec<PostModel>> {
        Ok(self
            .repository
            .select()
            .filter(PostColumn::Status.eq("0"))
            .order_by_asc(PostColumn::PostSort)
            .all(self.db.as_ref())
            .await?)
    }
}
