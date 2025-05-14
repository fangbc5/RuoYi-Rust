use async_trait::async_trait;
use ruoyi_common::{error::Error, vo::PageParam, Result};
use sea_orm::{ActiveValue::NotSet, IntoActiveModel, Set};
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fmt::Debug, sync::Arc};

use crate::common::utils;
use crate::config::GenConfig;
use crate::entity::prelude::*;
use crate::repository::{GenTableColumnRepository, GenTableQuery, GenTableRepository};

/// 代码生成表服务接口
#[async_trait]
pub trait GenTableService: Send + Sync {
    /// 查询业务表列表
    async fn select_gen_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)>;

    /// 查询业务表详情
    async fn select_gen_table_by_id(&self, table_id: i64) -> Result<Option<GenTableModel>>;

    /// 查询业务表详情及其字段信息
    async fn select_gen_table_detail_by_id(&self, table_id: i64) -> Result<Option<GenTableDetail>>;

    /// 查询数据库表列表（模糊查询）
    async fn select_db_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)>;

    /// 导入表结构
    async fn import_gen_table(&self, table_names: Vec<&str>) -> Result<()>;

    /// 创建业务表
    async fn create_gen_table(&self, gen_table: GenTableModel) -> Result<()>;

    /// 修改业务表
    async fn update_gen_table(&self, request: UpdateGenTableRequest) -> Result<()>;

    /// 删除业务表
    async fn delete_gen_table_by_ids(&self, table_ids: Vec<i64>) -> Result<()>;

    /// 预览代码
    async fn preview_code(&self, table_id: i64) -> Result<Vec<TemplateData>>;

    /// 生成代码
    async fn generate_code(&self, table_id: i64) -> Result<Vec<u8>>;

    /// 同步数据库
    async fn synch_db(&self, table_id: i64) -> Result<()>;

    /// 批量生成代码
    async fn batch_gen_code(&self, table_id: i64) -> Result<()>;
}

/// 代码生成表详情（包含字段信息）
#[derive(Debug, Serialize)]
pub struct GenTableDetail {
    pub info: GenTableModel,
    pub rows: Vec<GenTableColumnModel>,
    pub tables: Vec<GenTableModel>,
}

/// 修改代码生成业务表请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGenTableRequest {
    // 表基本信息
    pub table_id: i64,
    // 列信息
    pub columns: Vec<GenTableColumnModel>,

    pub table_name: Option<String>,
    pub table_comment: Option<String>,
    pub sub_table_name: Option<String>,
    pub sub_table_fk_name: Option<String>,
    pub class_name: Option<String>,
    pub tpl_category: Option<String>,
    pub tpl_web_type: Option<String>,
    pub package_name: Option<String>,
    pub module_name: Option<String>,
    pub business_name: Option<String>,
    pub function_name: Option<String>,
    pub function_author: Option<String>,
    pub gen_type: Option<String>,
    pub gen_path: Option<String>,
    pub options: Option<String>,
    pub remark: Option<String>,

    #[serde(skip)]
    pub create_by: Option<String>,
    #[serde(skip)]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip)]
    pub update_by: Option<String>,
    #[serde(skip)]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    pub params: Option<HashMap<String, String>>,
}

/// 模板数据
#[derive(Debug, Serialize)]
pub struct TemplateData {
    pub template_name: String,
    pub content: String,
}

/// 代码生成表服务实现
pub struct GenTableServiceImpl {
    db: Arc<DatabaseConnection>,
    gen_table_repository: Arc<dyn GenTableRepository>,
    gen_table_column_repository: Arc<dyn GenTableColumnRepository>,
}

impl GenTableServiceImpl {
    pub fn new(
        db: Arc<DatabaseConnection>,
        gen_table_repository: Arc<dyn GenTableRepository>,
        gen_table_column_repository: Arc<dyn GenTableColumnRepository>,
    ) -> Self {
        Self {
            db,
            gen_table_repository,
            gen_table_column_repository,
        }
    }
}

#[async_trait]
impl GenTableService for GenTableServiceImpl {
    async fn select_gen_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)> {
        self.gen_table_repository
            .select_gen_table_list(page_param, query)
            .await
    }

    async fn select_gen_table_by_id(&self, table_id: i64) -> Result<Option<GenTableModel>> {
        self.gen_table_repository
            .select_gen_table_by_id(table_id)
            .await
    }

    async fn select_gen_table_detail_by_id(&self, table_id: i64) -> Result<Option<GenTableDetail>> {
        let table_info = self
            .gen_table_repository
            .select_gen_table_by_id(table_id)
            .await?;

        if let Some(info) = table_info {
            let rows = self
                .gen_table_column_repository
                .select_gen_table_column_list_by_table_id(table_id)
                .await?;
            let tables = self.gen_table_repository.select_gen_table_all().await?;
            Ok(Some(GenTableDetail { info, rows, tables }))
        } else {
            Ok(None)
        }
    }

    async fn select_db_table_list(
        &self,
        page_param: &PageParam,
        query: &GenTableQuery,
    ) -> Result<(Vec<GenTableModel>, u64)> {
        self.gen_table_repository
            .select_db_table_list(page_param, query)
            .await
    }

    async fn import_gen_table(&self, table_names: Vec<&str>) -> Result<()> {
        let tx = self.db.begin().await?;
        for table_name in table_names {
            // 查询表是否在数据库中存在
            let table_db = self
                .gen_table_repository
                .select_db_table_by_name(table_name)
                .await?;
            if table_db.is_none() {
                return Err(Error::BusinessError(format!(
                    "表[{}],在数据库中不存在",
                    table_name
                )));
            }
            // 查询表是否存在
            let table_info = self
                .gen_table_repository
                .select_gen_table_by_name(table_name)
                .await?;
            if table_info.is_some() {
                return Err(Error::BusinessError(format!(
                    "表[{}],在库gen_table中已存在",
                    table_name
                )));
            }
            // 创建并插入表
            let table_db = table_db.unwrap();
            let gen_config = Arc::new(GenConfig::default());
            let gen_table = utils::init_table(gen_config.clone(), table_db).await;
            // 填充信息
            let gen_table = self
                .gen_table_repository
                .insert_gen_table(gen_table, &tx)
                .await?;
            // 查询表的所有列信息
            let columns = self
                .gen_table_column_repository
                .select_db_table_column_by_name(table_name)
                .await?;
            println!("columns: {:?}", columns);
            for column in columns {
                let column = utils::init_table_column(&gen_table, column).await;
                self.gen_table_column_repository
                    .insert_gen_table_column(column, &tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    async fn create_gen_table(&self, gen_table: GenTableModel) -> Result<()> {
        Ok(())
    }

    async fn update_gen_table(&self, request: UpdateGenTableRequest) -> Result<()> {
        let tx = self.db.begin().await?;

        // 将请求中的表信息转换为 GenTableActiveModel
        let active_model = GenTableActiveModel {
            table_id: Set(request.table_id),
            table_name: Set(request.table_name),
            table_comment: Set(request.table_comment),
            sub_table_name: Set(request.sub_table_name),
            sub_table_fk_name: Set(request.sub_table_fk_name),
            class_name: Set(request.class_name),
            tpl_category: Set(request.tpl_category),
            tpl_web_type: Set(request.tpl_web_type),
            package_name: Set(request.package_name),
            module_name: Set(request.module_name),
            business_name: Set(request.business_name),
            function_name: Set(request.function_name),
            function_author: Set(request.function_author),
            gen_type: Set(request.gen_type),
            gen_path: Set(request.gen_path),
            options: Set(request.options),
            create_by: NotSet, // 不更新创建信息
            create_time: NotSet,
            update_by: NotSet,
            update_time: NotSet, // 设置为当前时间
            remark: Set(request.remark),
        };

        // 更新表信息
        self.gen_table_repository
            .update_gen_table(active_model, &tx)
            .await?;

        // 更新列信息
        for column in request.columns {
            let column_model = GenTableColumnActiveModel {
                column_id: Set(column.column_id),
                column_name: Set(column.column_name),
                column_comment: Set(column.column_comment),
                column_type: Set(column.column_type),
                java_type: Set(column.java_type),
                java_field: Set(column.java_field),
                is_pk: Set(column.is_pk),
                is_increment: Set(column.is_increment),
                is_required: Set(column.is_required),
                is_insert: Set(column.is_insert),
                is_edit: Set(column.is_edit),
                is_list: Set(column.is_list),
                is_query: Set(column.is_query),
                query_type: Set(column.query_type),
                html_type: Set(column.html_type),
                dict_type: Set(column.dict_type),
                sort: Set(column.sort),
                create_by: NotSet,
                create_time: NotSet,
                update_by: NotSet,
                update_time: NotSet,
                table_id: NotSet,
            };
            self.gen_table_column_repository
                .update_gen_table_column(column_model, &tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_gen_table_by_ids(&self, table_ids: Vec<i64>) -> Result<()> {
        let tx = self.db.begin().await?;
        // 删除表
        self.gen_table_repository
            .delete_gen_table_by_ids(table_ids.clone(), &tx)
            .await?;

        // 删除表字段
        self.gen_table_column_repository
            .delete_gen_table_column_by_table_ids(&table_ids, &tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn preview_code(&self, table_id: i64) -> Result<Vec<TemplateData>> {
        // 获取表信息
        let table_detail = self.select_gen_table_detail_by_id(table_id).await?;

        if let Some(detail) = table_detail {
            // TODO: 实现代码生成逻辑
            // 这里简单返回一个示例
            let mut templates = Vec::new();

            let table_comment = detail.info.table_comment.clone().unwrap_or_default();
            let class_name = detail.info.class_name.clone().unwrap_or_default();

            templates.push(TemplateData {
                template_name: "Entity.java".to_string(),
                content: format!(
                    "// {} Entity\npublic class {} {{\n    // TODO: 实现实体类\n}}",
                    table_comment, class_name
                ),
            });

            templates.push(TemplateData {
                template_name: "Repository.java".to_string(),
                content: format!("// {} Repository\npublic interface {}Repository {{\n    // TODO: 实现仓库接口\n}}", 
                    table_comment,
                    class_name)
            });

            Ok(templates)
        } else {
            Err(Error::NotFound("表不存在".to_string()))
        }
    }

    async fn generate_code(&self, table_id: i64) -> Result<Vec<u8>> {
        // 获取表信息
        todo!()
    }

    async fn synch_db(&self, table_id: i64) -> Result<()> {
        todo!()
    }

    async fn batch_gen_code(&self, table_id: i64) -> Result<()> {
        todo!()
    }
}

// 辅助函数
fn table_name_to_class_name(table_name: &str) -> Option<String> {
    let parts: Vec<&str> = table_name.split('_').collect();
    if parts.len() > 1 {
        let mut class_name = String::new();
        // 从第二部分开始（跳过前缀）
        for part in parts.iter().skip(1) {
            if !part.is_empty() {
                let mut chars: Vec<char> = part.chars().collect();
                if let Some(first) = chars.first_mut() {
                    *first = first.to_uppercase().next().unwrap_or(*first);
                }
                class_name.push_str(&chars.into_iter().collect::<String>());
            }
        }
        Some(class_name)
    } else {
        // 如果没有下划线，直接首字母大写
        let mut chars: Vec<char> = table_name.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        Some(chars.into_iter().collect::<String>())
    }
}

fn get_module_name(table_name: &str) -> Option<String> {
    let parts: Vec<&str> = table_name.split('_').collect();
    if parts.len() > 1 {
        Some(parts[0].to_string())
    } else {
        Some("system".to_string())
    }
}

fn get_business_name(table_name: &str) -> Option<String> {
    let parts: Vec<&str> = table_name.split('_').collect();
    if parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        Some(table_name.to_string())
    }
}

fn column_name_to_java_field(column_name: &str) -> String {
    let parts: Vec<&str> = column_name.split('_').collect();
    let mut java_field = String::new();

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            java_field.push_str(part);
        } else if !part.is_empty() {
            let mut chars: Vec<char> = part.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            java_field.push_str(&chars.into_iter().collect::<String>());
        }
    }

    java_field
}
