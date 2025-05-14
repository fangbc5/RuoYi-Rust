use async_trait::async_trait;
use ruoyi_common::Result;
use std::sync::Arc;

use crate::entity::prelude::*;
use crate::repository::GenTableColumnRepository;

/// 代码生成表字段服务接口
#[async_trait]
pub trait GenTableColumnService: Send + Sync {
    /// 查询表字段列表
    async fn select_gen_table_column_list_by_table_id(
        &self,
        table_id: i64,
    ) -> Result<Vec<GenTableColumnModel>>;

    /// 查询表字段详情
    async fn select_gen_table_column_by_id(
        &self,
        column_id: i64,
    ) -> Result<Option<GenTableColumnModel>>;

    /// 新增表字段
    async fn insert_gen_table_column(&self, gen_table_column: GenTableColumnModel) -> Result<i64>;

    /// 修改表字段
    async fn update_gen_table_column(&self, gen_table_column: GenTableColumnModel) -> Result<()>;

    /// 批量修改表字段
    async fn update_gen_table_column_batch(
        &self,
        table_id: i64,
        columns: Vec<GenTableColumnModel>,
    ) -> Result<()>;

    /// 删除表字段
    async fn delete_gen_table_column_by_ids(&self, ids: &[i64]) -> Result<()>;
}

/// 代码生成表字段服务实现
pub struct GenTableColumnServiceImpl {
    gen_table_column_repository: Arc<dyn GenTableColumnRepository>,
}

impl GenTableColumnServiceImpl {
    pub fn new(gen_table_column_repository: Arc<dyn GenTableColumnRepository>) -> Self {
        Self {
            gen_table_column_repository,
        }
    }
}

#[async_trait]
impl GenTableColumnService for GenTableColumnServiceImpl {
    async fn select_gen_table_column_list_by_table_id(
        &self,
        table_id: i64,
    ) -> Result<Vec<GenTableColumnModel>> {
        self.gen_table_column_repository
            .select_gen_table_column_list_by_table_id(table_id)
            .await
    }

    async fn select_gen_table_column_by_id(
        &self,
        column_id: i64,
    ) -> Result<Option<GenTableColumnModel>> {
        // 这个方法在仓库层没有实现，需要扩展仓库层或者在这里实现
        // 暂时返回空实现
        Ok(None)
    }

    async fn insert_gen_table_column(&self, gen_table_column: GenTableColumnModel) -> Result<i64> {
        self.gen_table_column_repository
            .insert_gen_table_column(gen_table_column)
            .await
    }

    async fn update_gen_table_column(&self, gen_table_column: GenTableColumnModel) -> Result<()> {
        self.gen_table_column_repository
            .update_gen_table_column(gen_table_column)
            .await
    }

    async fn update_gen_table_column_batch(
        &self,
        table_id: i64,
        columns: Vec<GenTableColumnModel>,
    ) -> Result<()> {
        // 批量更新字段
        for column in columns {
            // 确保字段属于指定的表
            if column.table_id.unwrap_or(-1) == table_id {
                self.update_gen_table_column(column).await?;
            }
        }
        Ok(())
    }

    async fn delete_gen_table_column_by_ids(&self, ids: &[i64]) -> Result<()> {
        self.gen_table_column_repository
            .delete_gen_table_column_by_ids(ids)
            .await
    }
}
