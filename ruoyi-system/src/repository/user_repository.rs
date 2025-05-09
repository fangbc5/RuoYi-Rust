// ruoyi-system/src/repository/user_repository.rs
//! 用户仓库实现

use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::Result;
use ruoyi_common::{error::Error, vo::PageParam};
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::sync::Arc;

use crate::{controller::user_controller::UserQuery, entity::prelude::*};

/// 用户仓库特征
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 根据用户ID查找用户
    async fn find_by_id(&self, user_id: i64) -> Result<Option<UserModel>>;

    /// 根据用户名查找用户
    async fn find_by_username(&self, username: &str) -> Result<Option<UserModel>>;

    /// 创建用户和关联角色
    async fn create_user(
        &self,
        user_active_model: UserActiveModel,
        role_ids: Option<Vec<i64>>,
        post_ids: Option<Vec<i64>>,
    ) -> Result<UserModel>;

    /// 更新用户和关联角色
    async fn update_user(
        &self,
        user_active_model: UserActiveModel,
        role_ids: Option<Vec<i64>>,
        post_ids: Option<Vec<i64>>,
    ) -> Result<UserModel>;

    /// 批量删除用户
    async fn delete_users_by_ids(&self, user_ids: Vec<i64>) -> Result<u64>;

    /// 重置用户密码
    async fn reset_password(&self, user_id: i64, password: &str) -> Result<UserModel>;

    /// 更新用户状态
    async fn update_user_status(&self, user_id: i64, status: &str) -> Result<UserModel>;

    /// 检查用户名是否唯一
    async fn check_user_name_unique(
        &self,
        username: &str,
        exclude_user_id: Option<i64>,
    ) -> Result<bool>;

    /// 检查手机号是否唯一
    async fn check_phone_unique(&self, phone: &str, exclude_user_id: Option<i64>) -> Result<bool>;

    /// 检查邮箱是否唯一
    async fn check_email_unique(&self, email: &str, exclude_user_id: Option<i64>) -> Result<bool>;

    /// 查询用户列表
    async fn find_user_list(
        &self,
        query: &UserQuery,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)>;
}

/// 用户仓库实现
pub struct UserRepositoryImpl {
    /// 基础仓库
    repository: BaseRepository<UserEntity, UserActiveModel>,
    /// 数据库连接
    db: Arc<DatabaseConnection>,
}

impl UserRepositoryImpl {
    /// 创建用户仓库
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            repository: BaseRepository::new(db.as_ref().clone()),
            db,
        }
    }

    /// 构建查询条件
    fn build_query_condition(&self, query: &UserQuery) -> Condition {
        let mut condition = Condition::all();

        if let Some(user_id) = query.user_id {
            condition = condition.add(UserColumn::UserId.eq(user_id));
        }

        if let Some(dept_id) = query.dept_id {
            condition = condition.add(UserColumn::DeptId.eq(dept_id));
        }

        if let Some(ref user_name) = query.user_name {
            if !user_name.is_empty() {
                condition = condition.add(UserColumn::UserName.contains(user_name));
            }
        }

        if let Some(ref nick_name) = query.nick_name {
            if !nick_name.is_empty() {
                condition = condition.add(UserColumn::NickName.contains(nick_name));
            }
        }

        if let Some(ref phonenumber) = query.phonenumber {
            if !phonenumber.is_empty() {
                condition = condition.add(UserColumn::Phonenumber.contains(phonenumber));
            }
        }

        if let Some(ref status) = query.status {
            if !status.is_empty() {
                condition = condition.add(UserColumn::Status.eq(status));
            }
        }

        if let Some(ref begin_time) = query.begin_time {
            condition = condition.add(UserColumn::CreateTime.gt(begin_time.clone()));
        }

        if let Some(ref end_time) = query.end_time {
            condition = condition.add(UserColumn::CreateTime.lt(end_time.clone()));
        }

        // 默认只查询未删除的用户
        condition = condition.add(UserColumn::DelFlag.eq("0"));

        condition
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, user_id: i64) -> Result<Option<UserModel>> {
        let user_opt = self.repository.find_by_id(user_id).await?;
        if user_opt.is_some() && user_opt.as_ref().unwrap().del_flag == Some("0".to_string()) {
            Ok(user_opt)
        } else {
            Ok(None)
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<UserModel>> {
        Ok(self
            .repository
            .select()
            .filter(UserColumn::UserName.eq(username))
            .filter(UserColumn::DelFlag.eq("0"))
            .one(self.db.as_ref())
            .await?)
    }

    async fn create_user(
        &self,
        mut user_active_model: UserActiveModel,
        role_ids: Option<Vec<i64>>,
        post_ids: Option<Vec<i64>>,
    ) -> Result<UserModel> {
        if let Some(user) = get_sync_user_context() {
            user_active_model.create_by = Set(Some(user.user_name.clone()));
            user_active_model.update_by = Set(Some(user.user_name.clone()));
        }
        let txn = self.db.begin().await?;
        // 设置创建信息
        let now = Utc::now();
        user_active_model.create_time = Set(Some(now));
        user_active_model.update_time = Set(Some(now));

        // 标记为未删除
        user_active_model.del_flag = Set(Some("0".to_string()));

        // 插入用户
        let user = user_active_model.insert(&txn).await?;

        // 添加用户角色关联
        if let Some(role_ids) = role_ids {
            for role_id in role_ids {
                let user_role = UserRoleActiveModel {
                    user_id: Set(user.user_id),
                    role_id: Set(role_id),
                };
                user_role.insert(&txn).await?;
            }
        }

        // 添加用户岗位关联
        if let Some(post_ids) = post_ids {
            for post_id in post_ids {
                let user_post = UserPostActiveModel {
                    user_id: Set(user.user_id),
                    post_id: Set(post_id),
                };
                user_post.insert(&txn).await?;
            }
        }

        txn.commit().await?;
        Ok(user)
    }

    async fn update_user(
        &self,
        mut user_active_model: UserActiveModel,
        role_ids: Option<Vec<i64>>,
        post_ids: Option<Vec<i64>>,
    ) -> Result<UserModel> {
        if let Some(user) = get_sync_user_context() {
            user_active_model.update_by = Set(Some(user.user_name.clone()));
        }
        let txn = self.db.begin().await?;
        let user_id = user_active_model.user_id.clone().unwrap();
        // 设置更新时间
        user_active_model.update_time = Set(Some(Utc::now()));
        // 删除旧的角色关联
        UserRoleEntity::delete_many()
            .filter(UserRoleColumn::UserId.eq(user_id))
            .exec(&txn)
            .await?;
        // 添加新的角色关联
        if let Some(role_ids) = role_ids {
            for role_id in role_ids {
                let user_role = UserRoleActiveModel {
                    user_id: Set(user_id),
                    role_id: Set(role_id),
                };
                user_role.insert(&txn).await?;
            }
        }
        // 删除旧的岗位关联
        UserPostEntity::delete_many()
            .filter(UserPostColumn::UserId.eq(user_id))
            .exec(&txn)
            .await?;
        // 添加新的岗位关联
        if let Some(post_ids) = post_ids {
            for post_id in post_ids {
                let user_post = UserPostActiveModel {
                    user_id: Set(user_id),
                    post_id: Set(post_id),
                };
                user_post.insert(&txn).await?;
            }
        }
        // 更新用户
        let user = user_active_model.update(&txn).await?;

        txn.commit().await?;
        Ok(user)
    }

    async fn delete_users_by_ids(&self, user_ids: Vec<i64>) -> Result<u64> {
        let txn = self.db.begin().await?;
        // 软删除用户 (设置 del_flag = "2")
        let mut update = UserActiveModel {
            del_flag: Set(Some("2".to_string())),
            update_time: Set(Some(Utc::now())),
            ..Default::default()
        };
        if let Some(user) = get_sync_user_context() {
            update.update_by = Set(Some(user.user_name.clone()));
        }
        let res = UserEntity::update_many()
            .filter(UserColumn::UserId.is_in(user_ids.clone()))
            .set(update)
            .exec(&txn)
            .await?;

        // 删除用户角色关联
        UserRoleEntity::delete_many()
            .filter(UserRoleColumn::UserId.is_in(user_ids))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(res.rows_affected)
    }

    async fn reset_password(&self, user_id: i64, password: &str) -> Result<UserModel> {
        // 查询用户
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("用户不存在: {}", user_id)))?;

        // 更新密码
        let mut active_model: UserActiveModel = user.clone().into_active_model();
        active_model.password = Set(Some(password.to_string()));
        active_model.update_time = Set(Some(Utc::now()));
        if let Some(user) = get_sync_user_context() {
            active_model.update_by = Set(Some(user.user_name.clone()));
        }

        let updated_user = active_model.update(self.db.as_ref()).await?;
        Ok(updated_user)
    }

    async fn update_user_status(&self, user_id: i64, status: &str) -> Result<UserModel> {
        // 查询用户
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("用户不存在: {}", user_id)))?;

        // 更新状态
        let mut active_model: UserActiveModel = user.clone().into_active_model();
        active_model.status = Set(Some(status.to_string()));
        active_model.update_time = Set(Some(Utc::now()));
        if let Some(user) = get_sync_user_context() {
            active_model.update_by = Set(Some(user.user_name.clone()));
        }
        let updated_user = active_model.update(self.db.as_ref()).await?;
        Ok(updated_user)
    }

    async fn check_user_name_unique(
        &self,
        username: &str,
        exclude_user_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = UserEntity::find()
            .filter(UserColumn::UserName.eq(username))
            .filter(UserColumn::DelFlag.eq("0"));

        if let Some(user_id) = exclude_user_id {
            query = query.filter(UserColumn::UserId.ne(user_id));
        }

        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn check_phone_unique(&self, phone: &str, exclude_user_id: Option<i64>) -> Result<bool> {
        let mut query = UserEntity::find()
            .filter(UserColumn::Phonenumber.eq(phone))
            .filter(UserColumn::DelFlag.eq("0"));

        if let Some(user_id) = exclude_user_id {
            query = query.filter(UserColumn::UserId.ne(user_id));
        }

        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn check_email_unique(&self, email: &str, exclude_user_id: Option<i64>) -> Result<bool> {
        let mut query = UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .filter(UserColumn::DelFlag.eq("0"));

        if let Some(user_id) = exclude_user_id {
            query = query.filter(UserColumn::UserId.ne(user_id));
        }

        let count = query.count(self.db.as_ref()).await?;
        Ok(count == 0)
    }

    async fn find_user_list(
        &self,
        query: &UserQuery,
        page_param: &PageParam,
    ) -> Result<(Vec<UserModel>, u64)> {
        let condition = self.build_query_condition(query);

        let paginator = UserEntity::find()
            .filter(condition)
            .order_by(UserColumn::UserId, sea_orm::Order::Asc)
            .paginate(self.db.as_ref(), page_param.page_size as u64);

        let total = paginator.num_items().await?;
        let users = paginator.fetch_page(page_param.page_num as u64 - 1).await?;

        Ok((users, total))
    }
}
