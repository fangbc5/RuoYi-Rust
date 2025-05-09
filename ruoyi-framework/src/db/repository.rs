// ruoyi-framework/src/db/repository.rs
//! 数据库仓库模块，提供通用的 CRUD 操作接口

use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, Select, TransactionTrait,
};
use std::fmt::Debug;
use std::str::FromStr;

use ruoyi_common::vo::PageParam;

/// 通用仓库特征
#[async_trait::async_trait]
pub trait Repository<E, A>: Send + Sync
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync,
{
    /// 查找所有记录
    async fn find_all(&self) -> Result<Vec<E::Model>, DbErr>;

    /// 根据ID查找记录
    async fn find_by_id<T>(&self, id: T) -> Result<Option<E::Model>, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync;

    /// 插入实体
    async fn insert(&self, active_model: A) -> Result<E::Model, DbErr>;

    /// 更新实体
    async fn update(&self, active_model: A) -> Result<E::Model, DbErr>;

    /// 根据模型删除实体
    async fn delete_by_model(&self, model: E::Model) -> Result<u64, DbErr>
    where
        E::Model: IntoActiveModel<A>;

    /// 根据主键删除实体
    async fn delete_by_id<T>(&self, id: T) -> Result<u64, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync;

    /// 根据主键列表删除实体
    async fn delete_by_ids<T>(&self, ids: Vec<T>) -> Result<u64, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync;

    /// 分页查询
    async fn paginate(
        &self,
        page_param: &PageParam,
        condition: Option<Condition>,
    ) -> Result<(Vec<E::Model>, u64), DbErr>;
}

/// 基础仓库实现
pub struct BaseRepository<E, A>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync,
{
    /// 数据库连接
    db: DatabaseConnection,
    /// 实体类型
    _phantom: std::marker::PhantomData<(E, A)>,
}

impl<E, A> BaseRepository<E, A>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync,
{
    /// 创建新的仓库实例
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 获取查询选择器
    pub fn select(&self) -> Select<E> {
        E::find()
    }
}

#[async_trait::async_trait]
impl<E, A> Repository<E, A> for BaseRepository<E, A>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync,
    E::Model: Debug + Send + Sync + IntoActiveModel<A>,
    E::Column: FromStr + Debug,
    <E::Column as FromStr>::Err: Debug,
{
    async fn find_all(&self) -> Result<Vec<E::Model>, DbErr> {
        self.select().all(&self.db).await
    }

    async fn find_by_id<T>(&self, id: T) -> Result<Option<E::Model>, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync,
    {
        E::find_by_id(id).one(&self.db).await
    }

    async fn insert(&self, active_model: A) -> Result<E::Model, DbErr> {
        active_model.insert(&self.db).await
    }

    async fn update(&self, active_model: A) -> Result<E::Model, DbErr> {
        active_model.update(&self.db).await
    }

    async fn delete_by_model(&self, model: E::Model) -> Result<u64, DbErr>
    where
        E::Model: IntoActiveModel<A>,
    {
        model.delete(&self.db).await.map(|res| res.rows_affected)
    }

    async fn delete_by_id<T>(&self, id: T) -> Result<u64, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync,
    {
        E::delete_by_id(id)
            .exec(&self.db)
            .await
            .map(|res| res.rows_affected)
    }

    async fn delete_by_ids<T>(&self, ids: Vec<T>) -> Result<u64, DbErr>
    where
        T: Into<<<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>
            + Send
            + Sync,
    {
        let tx = self.db.begin().await?;
        let mut total = 0;
        for id in ids {
            let rows_affected = E::delete_by_id(id)
                .exec(&tx)
                .await
                .map(|res| res.rows_affected)
                .unwrap_or(0);
            total += rows_affected;
        }
        tx.commit().await?;
        Ok(total)
    }

    async fn paginate(
        &self,
        page_param: &PageParam,
        condition: Option<Condition>,
    ) -> Result<(Vec<E::Model>, u64), DbErr> {
        // 构建查询
        let mut query = self.select();
        if let Some(condition) = condition {
            query = query.filter(condition);
        }
        // 排序
        if let Some(order_by_column) = &page_param.order_by_column {
            if let Ok(column) = E::Column::from_str(order_by_column) {
                if let Some(is_asc) = &page_param.is_asc {
                    if is_asc.to_uppercase() == "ASC" {
                        query = query.order_by_asc(column);
                    } else {
                        query = query.order_by_desc(column);
                    }
                }
            }
        }

        // 创建分页器
        let paginator = query.paginate(&self.db, page_param.page_size);

        // 获取总记录数
        let total = paginator.num_items().await?;

        // 获取当前页数据
        let items = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((items, total))
    }
}
