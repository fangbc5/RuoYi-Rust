use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ruoyi_common::{
    utils::{self, time::deserialize_optional_datetime},
    vo::PageParam,
    Result,
};
use ruoyi_framework::{
    db::repository::{BaseRepository, Repository},
    web::tls::get_sync_user_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder,
    Set, Statement,
};
use serde::Deserialize;
use std::{str::FromStr, sync::Arc};

use crate::entity::prelude::*;

/// 代码生成表格仓库接口
#[async_trait]
pub trait GenTableRepository: Send + Sync {
    /// 查询业务表列表
    async fn select_gen_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)>;

    /// 查询业务表
    async fn select_gen_table_by_id(&self, table_id: i64) -> Result<Option<GenTableModel>>;

    /// 查询业务表
    async fn select_gen_table_by_name(&self, table_name: &str) -> Result<Option<GenTableModel>>;

    /// 查询所有业务表
    async fn select_gen_table_all(&self) -> Result<Vec<GenTableModel>>;

    /// 查询指定数据库表列表
    async fn select_db_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)>;

    /// 根据表名称查询数据库表
    async fn select_db_table_by_name(&self, table_name: &str) -> Result<Option<GenTableModel>>;

    /// 插入业务表
    async fn insert_gen_table(
        &self,
        mut gen_table: GenTableActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableModel>;

    /// 更新业务表
    async fn update_gen_table(&self, mut gen_table: GenTableActiveModel, tx: &DatabaseTransaction) -> Result<GenTableModel>;

    /// 批量删除业务表
    async fn delete_gen_table_by_ids(&self, table_ids: Vec<i64>, tx: &DatabaseTransaction) -> Result<u64>;
}

/// 代码生成表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenTableQuery {
    pub table_name: Option<String>,
    pub table_comment: Option<String>,

    /// 开始时间
    #[serde(rename = "params[beginTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub begin_time: Option<DateTime<Utc>>,

    /// 结束时间
    #[serde(rename = "params[endTime]", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub end_time: Option<DateTime<Utc>>,
}

/// 代码生成表格仓库实现
pub struct GenTableRepositoryImpl {
    repository: BaseRepository<GenTableEntity, GenTableActiveModel>,
    db: Arc<DatabaseConnection>,
}

impl GenTableRepositoryImpl {
    /// 创建代码生成表格仓库
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            repository: BaseRepository::new(db.as_ref().clone()),
            db,
        }
    }

    /// 构建查询条件
    fn build_query_condition(&self, query: &GenTableQuery) -> Condition {
        let mut condition = Condition::all();

        if let Some(ref table_name) = query.table_name {
            if !table_name.is_empty() {
                condition = condition.add(GenTableColumn::TableName.contains(table_name));
            }
        }

        if let Some(ref table_comment) = query.table_comment {
            if !table_comment.is_empty() {
                condition = condition.add(GenTableColumn::TableComment.contains(table_comment));
            }
        }

        if let Some(begin_time) = query.begin_time {
            condition = condition.add(GenTableColumn::CreateTime.gte(begin_time));
        }

        if let Some(end_time) = query.end_time {
            condition = condition.add(GenTableColumn::CreateTime.lte(end_time));
        }

        condition
    }
}

#[async_trait]
impl GenTableRepository for GenTableRepositoryImpl {
    async fn select_gen_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)> {
        let condition = self.build_query_condition(query);
        let mut query = self.repository.select();
        // 过滤条件
        query = query.filter(condition);
        // 排序
        if let Some(order_by_column) = &page_param.order_by_column {
            let order_by_column = utils::string::to_snake_case(&order_by_column);
            if page_param.is_asc == Some("ascending".to_string()) {
                query = query.order_by_asc(GenTableColumn::from_str(&order_by_column).unwrap());
            } else {
                query = query.order_by_desc(GenTableColumn::from_str(&order_by_column).unwrap());
            }
        }
        let paginator = query.paginate(self.db.as_ref(), page_param.page_size);

        let total = paginator.num_items().await?;
        let tables = paginator.fetch_page(page_param.page_num - 1).await?;

        Ok((tables, total))
    }

    async fn select_gen_table_by_id(&self, table_id: i64) -> Result<Option<GenTableModel>> {
        Ok(self.repository.find_by_id(table_id).await?)
    }

    async fn select_gen_table_by_name(&self, table_name: &str) -> Result<Option<GenTableModel>> {
        Ok(self
            .repository
            .select()
            .filter(GenTableColumn::TableName.eq(table_name))
            .one(self.db.as_ref())
            .await?)
    }

    async fn select_gen_table_all(&self) -> Result<Vec<GenTableModel>> {
        Ok(self.repository.select().all(self.db.as_ref()).await?)
    }

    async fn select_db_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)> {
        // 查询所有表，排除已存在的表
        let sql = format!(
            "SELECT table_name AS `table_name`, table_comment AS `table_comment`, 
            create_time AS `create_time`, update_time AS `update_time` 
            FROM information_schema.tables
            WHERE table_schema = (SELECT DATABASE()) 
            AND table_name NOT LIKE 'qrtz_%' AND table_name NOT LIKE 'gen_%'
            AND table_name NOT IN (SELECT table_name FROM gen_table)
            {}{}
            ORDER BY create_time DESC",
            if let Some(table_name) = &query.table_name {
                if !table_name.is_empty() {
                    format!(" AND table_name LIKE '%{}%'", table_name)
                } else {
                    String::new()
                }
            } else {
                String::new()
            },
            if let Some(table_comment) = &query.table_comment {
                if !table_comment.is_empty() {
                    format!(" AND table_comment LIKE '%{}%'", table_comment)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        );

        // 执行分页查询
        let count_sql = format!(
            "SELECT COUNT(*) as count
            FROM information_schema.tables
            WHERE table_schema = (SELECT DATABASE()) 
            AND table_name NOT LIKE 'qrtz_%' AND table_name NOT LIKE 'gen_%'
            AND table_name NOT IN (SELECT table_name FROM gen_table)
            {}{}",
            if let Some(table_name) = &query.table_name {
                if !table_name.is_empty() {
                    format!(" AND table_name LIKE '%{}%'", table_name)
                } else {
                    String::new()
                }
            } else {
                String::new()
            },
            if let Some(table_comment) = &query.table_comment {
                if !table_comment.is_empty() {
                    format!(" AND table_comment LIKE '%{}%'", table_comment)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        );

        // 计算总数
        #[derive(FromQueryResult)]
        struct CountResult {
            count: i64,
        }

        let count_stmt = Statement::from_string(DatabaseBackend::MySql, count_sql);
        let count_result = CountResult::find_by_statement(count_stmt)
            .one(self.db.as_ref())
            .await?;

        let total = count_result.map_or(0, |result| result.count as u64);

        // 查询分页数据
        let page_sql = format!(
            "{} LIMIT {} OFFSET {}",
            sql,
            page_param.page_size,
            (page_param.page_num - 1) * page_param.page_size
        );
        // 为 information_schema.tables 创建自定义结构
        #[derive(FromQueryResult, Debug)]
        struct DbTable {
            table_name: Option<String>,
            table_comment: Option<String>,
            create_time: Option<DateTime<Utc>>,
            update_time: Option<DateTime<Utc>>,
        }

        let stmt = Statement::from_string(DatabaseBackend::MySql, page_sql);
        let db_tables = DbTable::find_by_statement(stmt)
            .all(self.db.as_ref())
            .await?;

        // 转换为 GenTableModel
        let tables: Vec<GenTableModel> = db_tables
            .into_iter()
            .map(|db_table| GenTableModel {
                table_id: 0, // 默认值，因为这是数据库中未导入的表
                table_name: db_table.table_name,
                table_comment: db_table.table_comment,
                sub_table_name: None,
                sub_table_fk_name: None,
                class_name: None,
                tpl_category: None,
                tpl_web_type: None,
                package_name: None,
                module_name: None,
                business_name: None,
                function_name: None,
                function_author: None,
                gen_type: None,
                gen_path: None,
                parent_menu_id: None,
                options: None,
                create_by: None,
                create_time: db_table.create_time,
                update_by: None,
                update_time: db_table.update_time,
                remark: None,
            })
            .collect();

        Ok((tables, total))
    }

    async fn select_db_table_by_name(&self, table_name: &str) -> Result<Option<GenTableModel>> {
        let sql = format!(
            "SELECT table_name as `table_name`, table_comment as `table_comment`, create_time as `create_time`, update_time as `update_time` 
            FROM information_schema.tables
            WHERE table_schema = (SELECT DATABASE()) 
            AND table_name = '{}'",
            table_name
        );

        // 为 information_schema.tables 创建自定义结构
        #[derive(FromQueryResult, Debug)]
        struct DbTable {
            table_name: Option<String>,
            table_comment: Option<String>,
            create_time: Option<DateTime<Utc>>,
            update_time: Option<DateTime<Utc>>,
        }

        let stmt = Statement::from_string(DatabaseBackend::MySql, sql);
        let db_table = DbTable::find_by_statement(stmt)
            .one(self.db.as_ref())
            .await?;

        if db_table.is_none() {
            return Ok(None);
        }
        // 转换为 GenTableModel
        let db_table = db_table.unwrap();
        let table = GenTableModel {
            table_id: 0, // 默认值，因为这是数据库中未导入的表
            table_name: db_table.table_name,
            table_comment: db_table.table_comment,
            sub_table_name: None,
            sub_table_fk_name: None,
            class_name: None,
            tpl_category: None,
            tpl_web_type: None,
            package_name: None,
            module_name: None,
            business_name: None,
            function_name: None,
            function_author: None,
            gen_type: None,
            gen_path: None,
            parent_menu_id: None,
            options: None,
            create_by: None,
            create_time: db_table.create_time,
            update_by: None,
            update_time: db_table.update_time,
            remark: None,
        };

        Ok(Some(table))
    }

    async fn insert_gen_table(
        &self,
        mut gen_table: GenTableActiveModel,
        tx: &DatabaseTransaction,
    ) -> Result<GenTableModel> {
        if let Some(user_context) = get_sync_user_context() {
            gen_table.create_by = Set(Some(user_context.user_name.clone()));
            gen_table.update_by = Set(Some(user_context.user_name.clone()));
        }
        gen_table.create_time = Set(Some(Utc::now()));
        gen_table.update_time = Set(Some(Utc::now()));
        Ok(gen_table.insert(tx).await?)
    }

    async fn update_gen_table(&self, mut gen_table: GenTableActiveModel, tx: &DatabaseTransaction) -> Result<GenTableModel> {
        if let Some(user_context) = get_sync_user_context() {
            gen_table.update_by = Set(Some(user_context.user_name.clone()));
        }
        gen_table.update_time = Set(Some(Utc::now()));
        let table = gen_table.update(tx).await?;
        Ok(table)
    }

    async fn delete_gen_table_by_ids(&self, table_ids: Vec<i64>, tx: &DatabaseTransaction) -> Result<u64> {
        let res = GenTableEntity::delete_many()
            .filter(GenTableColumn::TableId.is_in(table_ids))
            .exec(tx)
            .await?;

        Ok(res.rows_affected)
    }
}
