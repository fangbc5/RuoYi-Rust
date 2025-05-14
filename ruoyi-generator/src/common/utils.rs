use std::sync::Arc;

use regex::Regex;
use ruoyi_common::utils;
use sea_orm::{IntoActiveModel, Set};

use crate::{common::constants::*, config::GenConfig, entity::prelude::*};

pub async fn init_table(
    gen_config: Arc<GenConfig>,
    gen_table: GenTableModel,
) -> GenTableActiveModel {
    let table_name = gen_table.table_name.clone();
    let table_comment = gen_table.table_comment.clone();
    let mut gen_table = gen_table.into_active_model();
    if let Some(table_name) = table_name {
        gen_table.class_name = Set(Some(convert_class_name(&table_name, gen_config.clone())));
        gen_table.business_name = Set(Some(get_business_name(&table_name)));
    }
    if let Some(package_name) = gen_config.package_name.clone() {
        gen_table.package_name = Set(Some(package_name.clone()));
        gen_table.module_name = Set(Some(get_module_name(&package_name)));
    }
    if let Some(table_comment) = table_comment {
        gen_table.function_name = Set(Some(replace_text(&table_comment)));
    }
    if let Some(author) = gen_config.author.clone() {
        gen_table.function_author = Set(Some(author.clone()));
    }
    gen_table
}

pub async fn init_table_column(
    gen_table: &GenTableModel,
    gen_table_column: GenTableColumnModel,
) -> GenTableColumnActiveModel {
    let is_pk = if let Some(is_pk) = gen_table_column.is_pk.as_ref() {
        is_pk == "1"
    } else {
        false
    };
    let column_name = gen_table_column.column_name.clone();
    let column_type = gen_table_column.column_type.clone();
    let is_required: bool = if let Some(is_required) = gen_table_column.is_required.as_ref() {
        is_required == "1"
    } else {
        false
    };
    let mut gen_table_column = gen_table_column.into_active_model();
    gen_table_column.table_id = Set(Some(gen_table.table_id));
    if let Some(column_type) = column_type {
        gen_table_column.java_type = Set(Some(convert_column_type(&column_type, is_required)));
        let db_type = get_db_type(&column_type);
        if COLUMNTYPE_STR.contains(&db_type.as_str()) || COLUMNTYPE_TEXT.contains(&db_type.as_str())
        {
            let length = get_column_length(&column_type);
            let html_type = if length > 500 || COLUMNTYPE_TEXT.contains(&db_type.as_str()) {
                HTML_TEXTAREA
            } else {
                HTML_INPUT
            };
            gen_table_column.html_type = Set(Some(html_type.to_string()));
        } else if COLUMNTYPE_TIME.contains(&db_type.as_str()) {
            gen_table_column.java_type = Set(Some(TYPE_DATE.to_string()));
            gen_table_column.html_type = Set(Some(HTML_DATETIME.to_string()));
        } else if COLUMNTYPE_NUMBER.contains(&db_type.as_str()) {
            gen_table_column.html_type = Set(Some(HTML_INPUT.to_string()));

            // 如果是浮点型 统一用BigDecimal
            if let Some(between_brackets) = column_type.find('(').and_then(|start_idx| {
                column_type
                    .find(')')
                    .map(|end_idx| &column_type[(start_idx + 1)..end_idx])
            }) {
                let parts: Vec<&str> = between_brackets.split(',').collect();

                if parts.len() == 2 && parts[1].trim().parse::<i32>().map_or(false, |v| v > 0) {
                    // 浮点型
                    gen_table_column.java_type = Set(Some(TYPE_BIGDECIMAL.to_string()));
                } else if parts.len() == 1
                    && parts[0].trim().parse::<i32>().map_or(false, |v| v <= 10)
                {
                    // 整型
                    gen_table_column.java_type = Set(Some(TYPE_INTEGER.to_string()));
                } else {
                    // 长整型
                    gen_table_column.java_type = Set(Some(TYPE_LONG.to_string()));
                }
            } else {
                // 没有括号的情况，默认为长整型
                gen_table_column.java_type = Set(Some(TYPE_LONG.to_string()));
            }
        }
    }
    // 处理crud相关字段
    // 默认所有字段都要插入
    gen_table_column.is_insert = Set(Some(REQUIRE.to_string()));

    if let Some(column_name) = column_name.as_ref() {
        // 驼峰转下划线
        gen_table_column.java_field = Set(Some(utils::string::to_snake_case(column_name)));

        // 编辑字段
        if !COLUMNNAME_NOT_EDIT.contains(&column_name.as_str()) && !is_pk {
            gen_table_column.is_edit = Set(Some(REQUIRE.to_string()));
        }
        // 列表字段
        if !COLUMNNAME_NOT_LIST.contains(&column_name.as_str()) && !is_pk {
            gen_table_column.is_list = Set(Some(REQUIRE.to_string()));
        }
        // 查询字段
        if !COLUMNNAME_NOT_QUERY.contains(&column_name.as_str()) && !is_pk {
            gen_table_column.is_query = Set(Some(REQUIRE.to_string()));
        }

        // 查询字段类型
        if column_name.to_lowercase().ends_with("name") {
            gen_table_column.query_type = Set(Some(QUERY_LIKE.to_string()));
        }
        // 状态字段设置单选框
        if column_name.to_lowercase().ends_with("status") {
            gen_table_column.html_type = Set(Some(HTML_RADIO.to_string()));
        }
        // 类型&性别字段设置下拉框
        else if column_name.to_lowercase().ends_with("type")
            || column_name.to_lowercase().ends_with("sex")
        {
            gen_table_column.html_type = Set(Some(HTML_SELECT.to_string()));
        }
        // 图片字段设置图片上传控件
        else if column_name.to_lowercase().ends_with("image") {
            gen_table_column.html_type = Set(Some(HTML_IMAGE_UPLOAD.to_string()));
        }
        // 文件字段设置文件上传控件
        else if column_name.to_lowercase().ends_with("file") {
            gen_table_column.html_type = Set(Some(HTML_FILE_UPLOAD.to_string()));
        }
        // 内容字段设置富文本控件
        else if column_name.to_lowercase().ends_with("content") {
            gen_table_column.html_type = Set(Some(HTML_EDITOR.to_string()));
        }
    }

    gen_table_column
}

fn get_column_length(column_type: &str) -> usize {
    // 获取"("之后的内容
    if let Some(between_brackets) = column_type.find('(').and_then(|start_idx| {
        column_type
            .find(')')
            .map(|end_idx| &column_type[(start_idx + 1)..end_idx])
    }) {
        between_brackets.parse::<usize>().unwrap_or(0)
    } else {
        0
    }
}

fn get_db_type(column_type: &str) -> String {
    // 获取"("之前的内容
    let index = column_type.find('(').unwrap_or(column_type.len());
    column_type[..index].to_string()
}

fn convert_column_type(column_type: &str, is_required: bool) -> String {
    // 将mysql数据库中的类型转换为rust对应的类型
    let mut rust_type = match column_type {
        "int" => "i32".to_string(),
        "bigint" => "i64".to_string(),
        "varchar" => "String".to_string(),
        "datetime" => "DateTime<Utc>".to_string(),
        _ => "String".to_string(),
    };
    // 如果是非必填字段则包裹Option
    if !is_required {
        rust_type = format!("Option<{}>", rust_type);
    }
    rust_type
}

fn replace_text(text: &str) -> String {
    let regex = Regex::new(r"(?:表|若依)").unwrap();
    regex.replace(text, "").to_string()
}

fn get_module_name(package_name: &str) -> String {
    package_name.split('.').last().unwrap().to_string()
}

fn get_business_name(table_name: &str) -> String {
    table_name.split('_').last().unwrap().to_string()
}
fn convert_class_name(table_name: &str, gen_config: Arc<GenConfig>) -> String {
    let mut class_name = table_name.to_string();
    // 判断是否需要去除前缀
    if gen_config.auto_remove_pre {
        if let Some(prefix) = &gen_config.table_prefix {
            if table_name.starts_with(prefix) {
                class_name = table_name.replace(prefix, "");
            }
        }
    }
    // 将class_name转换为大驼峰命名
    utils::string::capitalize(&utils::string::to_camel_case(&class_name))
}
