use crate::controller::post_controller::{CreateOrUpdatePostRequest, PostQuery};
use crate::entity::prelude::*;
use crate::repository::post_repository::PostRepository;
use async_trait::async_trait;
use log::error;
use ruoyi_common::error::Error;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};
use std::sync::Arc;

#[async_trait]
pub trait PostService: Send + Sync {
    async fn get_post_list(
        &self,
        query: PostQuery,
        page_param: PageParam,
    ) -> Result<(Vec<PostModel>, u64)>;
    async fn get_post(&self, post_id: i64) -> Result<Option<PostModel>>;
    async fn create_post(&self, post: CreateOrUpdatePostRequest) -> Result<PostModel>;
    async fn update_post(&self, post: CreateOrUpdatePostRequest) -> Result<PostModel>;
    async fn delete_post(&self, post_ids: Vec<i64>) -> Result<u64>;
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
    async fn get_post_ids_by_user_id(&self, user_id: i64) -> Vec<i64>;
    async fn get_posts_by_user_id(&self, user_id: i64) -> Vec<PostModel>;
    async fn get_posts_all(&self) -> Vec<PostModel>;
}

pub struct PostServiceImpl {
    post_repository: Arc<dyn PostRepository>,
}

impl PostServiceImpl {
    pub fn new(post_repository: Arc<dyn PostRepository>) -> Self {
        Self { post_repository }
    }

    pub fn build_query_condition(&self, req: &PostQuery) -> Option<Condition> {
        let mut condition = Condition::all();
        if let Some(post_name) = &req.post_name {
            condition = condition.add(PostColumn::PostName.contains(post_name));
        }
        if let Some(post_code) = &req.post_code {
            condition = condition.add(PostColumn::PostCode.contains(post_code));
        }
        if let Some(status) = &req.status {
            condition = condition.add(PostColumn::Status.eq(status));
        }
        Some(condition)
    }
}

#[async_trait]
impl PostService for PostServiceImpl {
    async fn get_post_list(
        &self,
        req: PostQuery,
        page_param: PageParam,
    ) -> Result<(Vec<PostModel>, u64)> {
        Ok(self
            .post_repository
            .get_post_list(self.build_query_condition(&req), page_param)
            .await?)
    }

    async fn get_post(&self, post_id: i64) -> Result<Option<PostModel>> {
        Ok(self.post_repository.get_post_by_id(post_id).await?)
    }

    async fn create_post(&self, post: CreateOrUpdatePostRequest) -> Result<PostModel> {
        let model = PostModel {
            post_id: 0,
            post_code: post.post_code.unwrap_or_default(),
            post_name: post.post_name.unwrap_or_default(),
            post_sort: post.post_sort.unwrap_or(0),
            status: post.status.unwrap_or_default(),
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: post.remark,
        };

        Ok(self
            .post_repository
            .create_post(model.into_active_model())
            .await?)
    }

    async fn update_post(&self, post: CreateOrUpdatePostRequest) -> Result<PostModel> {
        let post_id = post.post_id.unwrap_or(0);
        let post_model = self.post_repository.get_post_by_id(post_id).await?;
        if post_model.is_none() {
            return Err(Error::BusinessError(format!("岗位不存在: {}", post_id)));
        }
        let mut active_model = post_model.unwrap().into_active_model();
        if let Some(post_code) = post.post_code {
            active_model.post_code = Set(post_code);
        }
        if let Some(post_name) = post.post_name {
            active_model.post_name = Set(post_name);
        }
        if let Some(post_sort) = post.post_sort {
            active_model.post_sort = Set(post_sort);
        }
        if let Some(status) = post.status {
            active_model.status = Set(status);
        }
        if let Some(remark) = post.remark {
            active_model.remark = Set(Some(remark));
        }

        Ok(self.post_repository.update_post(active_model).await?)
    }

    async fn delete_post(&self, post_ids: Vec<i64>) -> Result<u64> {
        Ok(self.post_repository.delete_posts(post_ids).await?)
    }

    async fn check_post_name_unique(
        &self,
        post_name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        Ok(self
            .post_repository
            .check_post_name_unique(post_name, exclude_id)
            .await?)
    }

    async fn check_post_code_unique(
        &self,
        post_code: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        Ok(self
            .post_repository
            .check_post_code_unique(post_code, exclude_id)
            .await?)
    }

    async fn get_posts_by_user_id(&self, user_id: i64) -> Vec<PostModel> {
        match self.post_repository.get_posts_by_user_id(user_id).await {
            Ok(posts) => posts,
            Err(e) => {
                error!(
                    "[post_service][get_posts_by_user_id]: 获取用户岗位失败: {}",
                    e
                );
                vec![]
            }
        }
    }

    async fn get_posts_all(&self) -> Vec<PostModel> {
        match self.post_repository.get_posts_all().await {
            Ok(posts) => posts,
            Err(e) => {
                error!("[post_service][get_posts_all]: 获取所有岗位失败: {}", e);
                vec![]
            }
        }
    }

    async fn get_post_ids_by_user_id(&self, user_id: i64) -> Vec<i64> {
        match self.post_repository.get_post_ids_by_user_id(user_id).await {
            Ok(post_ids) => post_ids,
            Err(e) => {
                error!(
                    "[post_service][get_post_ids_by_user_id]: 获取用户岗位ID失败: {}",
                    e
                );
                vec![]
            }
        }
    }
}
