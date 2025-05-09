// ruoyi-common/src/entity.rs
//! 通用实体定义

use sea_orm::{entity::prelude::*, Condition};

/// 通用的模型特征
#[async_trait::async_trait]
pub trait BaseModel: Sized {
    /// 模型ID类型
    type Id;

    /// 获取ID
    fn id(&self) -> Self::Id;

    /// 根据ID查询
    async fn find_by_id(&self, id: Self::Id, db: &DatabaseConnection) -> Result<Option<Self>, DbErr>;

    /// 查询全部
    async fn find_all(&self, db: &DatabaseConnection) -> Result<Vec<Self>, DbErr>;

    /// 根据条件查询
    async fn find_by_condition(&self, condition: &Condition, db: &DatabaseConnection) -> Result<Vec<Self>, DbErr>;

    /// 保存实体
    async fn save(&self, db: &DatabaseConnection) -> Result<Self::Id, DbErr>;

    /// 删除实体
    async fn delete(&self, db: &DatabaseConnection) -> Result<u64, DbErr>;

    /// 更新实体
    async fn update(&self, db: &DatabaseConnection) -> Result<Self, DbErr>;
}
