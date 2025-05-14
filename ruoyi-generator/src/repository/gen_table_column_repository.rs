use async_trait::async_trait;
use chrono::Utc;
use ruoyi_common::Result;
use ruoyi_framework::{db::repository::BaseRepository, web::tls::get_sync_user_context};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction,
    EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Set, Statement,
};
use std::sync::Arc;

use crate::entity::prelude::*;

/// 代码生成表字段仓库接口
#[async_trait]
pub trait GenTableColumnRepository: Send + Sync {
    /// 查询据库表字段信息
    async fn select_db_table_column_by_name(
        &self,
        table_name: &str,
    ) -> Result<Vec<GenTableColumnModel>>;

    /// 查询业务字段列表
    async fn select_gen_table_column_list_by_table_id(
        &self,
        table_id: i64,
    ) -> Result<Vec<GenTableColumnModel>>;

    /// 新增业务字段
    async fn insert_gen_table_column(
        &self,
        mut gen_table_column: GenTableColumnActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableColumnModel>;

    /// 修改业务字段
    async fn update_gen_table_column(
        &self,
        mut gen_table_column: GenTableColumnActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableColumnModel>;

    /// 删除业务字段
    async fn delete_gen_table_column_by_ids(&self, ids: &[i64]) -> Result<()>;

    /// 批量删除业务字段
    async fn delete_gen_table_column_by_table_ids(&self, table_ids: &[i64], tx: &DatabaseTransaction) -> Result<()>;
}

/// 代码生成表字段仓库实现
pub struct GenTableColumnRepositoryImpl {
    db: Arc<DatabaseConnection>,
    repository: BaseRepository<GenTableColumnEntity, GenTableColumnActiveModel>,
}

impl GenTableColumnRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            repository: BaseRepository::new((*db).clone()),
        }
    }
}

#[async_trait]
impl GenTableColumnRepository for GenTableColumnRepositoryImpl {
    async fn select_db_table_column_by_name(
        &self,
        table_name: &str,
    ) -> Result<Vec<GenTableColumnModel>> {
        let sql = format!("select column_name as `column_name`, (case when (is_nullable = 'no' and column_key != 'PRI') then '1' else '0' end) as is_required, (case when column_key = 'PRI' then '1' else '0' end) as is_pk, ordinal_position as sort, column_comment as `column_comment`, (case when extra = 'auto_increment' then '1' else '0' end) as is_increment, column_type as `column_type`
		from information_schema.columns where table_schema = (select database()) and table_name = ('{}')
		order by ordinal_position", table_name);
        // 为 information_schema.columns 创建自定义结构
        #[derive(FromQueryResult, Debug)]
        struct TableColumn {
            column_name: Option<String>,
            is_required: Option<String>,
            is_pk: Option<String>,
            sort: Option<u32>,
            column_comment: Option<String>,
            is_increment: Option<String>,
            column_type: Option<String>,
        }

        let columns =
            TableColumn::find_by_statement(Statement::from_string(DatabaseBackend::MySql, sql))
                .all(&*self.db)
                .await?;
        let columns = columns
            .into_iter()
            .map(|column| GenTableColumnModel {
                column_name: column.column_name,
                is_required: column.is_required,
                is_pk: column.is_pk,
                sort: column.sort.map(|s| s as i32),
                column_comment: column.column_comment,
                is_increment: column.is_increment,
                column_type: column.column_type,
                java_type: None,
                java_field: None,
                is_insert: None,
                is_edit: None,
                is_list: None,
                is_query: None,
                query_type: None,
                html_type: None,
                dict_type: None,
                create_by: None,
                create_time: None,
                update_by: None,
                update_time: None,
                table_id: None,
                column_id: 0,
            })
            .collect();
        Ok(columns)
    }

    async fn select_gen_table_column_list_by_table_id(
        &self,
        table_id: i64,
    ) -> Result<Vec<GenTableColumnModel>> {
        // 根据表ID查询业务字段列表的实现
        let columns = GenTableColumnEntity::find()
            .filter(GenTableColumnColumn::TableId.eq(table_id))
            .order_by_asc(GenTableColumnColumn::Sort)
            .all(&*self.db)
            .await?;
        Ok(columns)
    }

    async fn insert_gen_table_column(
        &self,
        mut gen_table_column: GenTableColumnActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableColumnModel> {
        // 插入业务字段的实现
        if let Some(user_context) = get_sync_user_context() {
            gen_table_column.create_by = Set(Some(user_context.user_name.clone()));
            gen_table_column.update_by = Set(Some(user_context.user_name.clone()));
        }
        gen_table_column.create_time = Set(Some(Utc::now()));
        gen_table_column.update_time = Set(Some(Utc::now()));
        Ok(gen_table_column.insert(tx).await?)
    }

    async fn update_gen_table_column(
        &self,
        mut gen_table_column: GenTableColumnActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableColumnModel> {
        // 更新业务字段的实现
        if let Some(user_context) = get_sync_user_context() {
            gen_table_column.update_by = Set(Some(user_context.user_name.clone()));
        }
        gen_table_column.update_time = Set(Some(Utc::now()));
        Ok(gen_table_column.update(tx).await?)
    }

    async fn delete_gen_table_column_by_ids(&self, ids: &[i64]) -> Result<()> {
        // 删除业务字段的实现
        GenTableColumnEntity::delete_many()
            .filter(GenTableColumnColumn::ColumnId.is_in(ids.to_vec()))
            .exec(&*self.db)
            .await?;
        Ok(())
    }

    async fn delete_gen_table_column_by_table_ids(&self, table_ids: &[i64], tx: &DatabaseTransaction) -> Result<()> {
        // 批量删除业务字段的实现
        GenTableColumnEntity::delete_many()
            .filter(GenTableColumnColumn::TableId.is_in(table_ids.to_vec()))
            .exec(tx)
            .await?;
        Ok(())
    }
}
