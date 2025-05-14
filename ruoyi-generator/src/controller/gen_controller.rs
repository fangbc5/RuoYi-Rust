use actix_web::{web, HttpResponse, Responder};
use log::error;
use ruoyi_common::vo::{PageParam, RData, R};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::entity::prelude::*;
use crate::repository::GenTableQuery;
use crate::service::gen_table_service::UpdateGenTableRequest;
use crate::service::{GenTableService, TemplateData};

/// 预览代码响应
#[derive(Debug, Serialize)]
pub struct PreviewCodeResponse {
    pub templates: Vec<TemplateData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TablesQuery {
    pub tables: String,
}

/// 代码生成控制器
pub struct GenController {
    gen_table_service: Arc<dyn GenTableService>,
}

impl GenController {
    pub fn new(gen_table_service: Arc<dyn GenTableService>) -> Self {
        Self { gen_table_service }
    }

    /// 查询代码生成列表
    pub async fn list(
        &self,
        page_param: web::Query<PageParam>,
        query: web::Query<GenTableQuery>,
    ) -> impl Responder {
        match self
            .gen_table_service
            .select_gen_table_list(&page_param, &query)
            .await
        {
            Ok((rows, total)) => HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "msg": "操作成功",
                "rows": rows,
                "total": total,
            })),
            Err(e) => {
                HttpResponse::InternalServerError().json(R::<()>::error_with_msg(&e.to_string()))
            }
        }
    }

    /// 查询数据库列表
    pub async fn db_list(
        &self,
        page_param: web::Query<PageParam>,
        query: web::Query<GenTableQuery>,
    ) -> impl Responder {
        match self
            .gen_table_service
            .select_db_table_list(&page_param, &query)
            .await
        {
            Ok((rows, total)) => HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "msg": "操作成功",
                "rows": rows,
                "total": total,
            })),
            Err(e) => {
                error!("查询数据库表失败: {}", e);
                HttpResponse::Ok().json(R::<()>::error_with_msg("查询数据库表失败"))
            }
        }
    }

    /// 查询表详细信息
    pub async fn get_info(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self
            .gen_table_service
            .select_gen_table_detail_by_id(table_id)
            .await
        {
            Ok(Some(detail)) => HttpResponse::Ok().json(RData::ok(detail)),
            Ok(None) => HttpResponse::Ok().json(R::<String>::fail("表不存在")),
            Err(e) => {
                error!("查询表详细信息失败: {}", e);
                HttpResponse::Ok().json(R::<()>::fail(&format!("查询表详细信息失败: {}", e)))
            }
        }
    }

    /// 创建表
    pub async fn create_table(&self, table: web::Json<GenTableModel>) -> impl Responder {
        match self
            .gen_table_service
            .create_gen_table(table.into_inner())
            .await
        {
            Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("创建成功")),
            Err(e) => {
                HttpResponse::InternalServerError().json(R::<()>::error_with_msg(&e.to_string()))
            }
        }
    }

    /// 修改代码生成业务
    pub async fn update(&self, table: web::Json<UpdateGenTableRequest>) -> impl Responder {
        let request = table.into_inner();
        log::info!(
            "收到更新请求，表ID: {}, 列数量: {}",
            request.table_id,
            request.columns.len()
        );
        log::debug!("请求数据: {:?}", request);

        match self.gen_table_service.update_gen_table(request).await {
            Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("修改成功")),
            Err(e) => {
                log::error!("修改失败: {:?}", e);
                HttpResponse::InternalServerError().json(R::<()>::error_with_msg(&e.to_string()))
            }
        }
    }

    /// 预览代码
    pub async fn preview(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self.gen_table_service.preview_code(table_id).await {
            Ok(templates) => {
                let response = PreviewCodeResponse { templates };
                HttpResponse::Ok().json(R::ok_with_data(response))
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(R::<()>::error_with_msg(&e.to_string()))
            }
        }
    }

    /// 生成代码（下载方式）
    pub async fn download(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self.gen_table_service.generate_code(table_id).await {
            Ok(data) => {
                // 返回ZIP文件
                HttpResponse::Ok()
                    .content_type("application/octet-stream")
                    .append_header(("Content-Disposition", "attachment; filename=\"ruoyi.zip\""))
                    .body(data)
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(R::<()>::error_with_msg(&e.to_string()))
            }
        }
    }

    /// 导入表结构
    pub async fn import_table(&self, query: web::Query<TablesQuery>) -> impl Responder {
        let tables = query.tables.split(',').collect::<Vec<&str>>();
        match self.gen_table_service.import_gen_table(tables).await {
            Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("导入成功")),
            Err(e) => {
                error!("导入表失败: {}", e);
                HttpResponse::Ok().json(R::<()>::error_with_msg("导入失败"))
            }
        }
    }

    /// 删除代码生成
    pub async fn delete(&self, ids: web::Path<String>) -> impl Responder {
        let id_vec: Result<Vec<i64>, _> = ids.split(',').map(|id| id.parse::<i64>()).collect();

        match id_vec {
            Ok(table_ids) => {
                match self
                    .gen_table_service
                    .delete_gen_table_by_ids(table_ids)
                    .await
                {
                    Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("删除成功")),
                    Err(e) => {
                        error!("删除代码生成失败: {}", e);
                        HttpResponse::Ok().json(R::<()>::error_with_msg("删除失败"))
                    }
                }
            }
            Err(_) => HttpResponse::BadRequest().json(R::<()>::error_with_msg("参数错误")),
        }
    }

    /// 生成代码
    pub async fn gen_code(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self.gen_table_service.generate_code(table_id).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Content-Disposition", "attachment; filename=\"ruoyi.zip\""))
                .body(data),
            Err(e) => {
                error!("生成代码失败: {}", e);
                HttpResponse::Ok().json(R::<()>::error_with_msg("生成失败"))
            }
        }
    }

    /// 同步数据库
    pub async fn synch_db(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self.gen_table_service.synch_db(table_id).await {
            Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("同步成功")),
            Err(e) => {
                error!("同步数据库失败: {}", e);
                HttpResponse::Ok().json(R::<()>::error_with_msg("同步失败"))
            }
        }
    }

    /// 批量生成代码
    pub async fn batch_gen_code(&self, path: web::Path<i64>) -> impl Responder {
        let table_id = path.into_inner();

        match self.gen_table_service.batch_gen_code(table_id).await {
            Ok(_) => HttpResponse::Ok().json(R::<()>::ok_with_msg("批量生成成功")),
            Err(e) => {
                error!("批量生成代码失败: {}", e);
                HttpResponse::Ok().json(R::<()>::error_with_msg("批量生成失败"))
            }
        }
    }
}

/// 配置代码生成路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tool/gen")
            // 表相关接口
            .route(
                "/list",
                web::get().to(
                    |gen: web::Data<GenController>,
                     page_param: web::Query<PageParam>,
                     query: web::Query<GenTableQuery>| async move {
                        gen.list(page_param, query).await
                    },
                ),
            )
            .route(
                "/{tableId}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.get_info(path).await
                    },
                ),
            )
            .route(
                "/db/list",
                web::get().to(
                    |gen: web::Data<GenController>,
                     page_param: web::Query<PageParam>,
                     query: web::Query<GenTableQuery>| async move {
                        gen.db_list(page_param, query).await
                    },
                ),
            )
            .route(
                "/column/{tableId}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.get_info(path).await
                    },
                ),
            )
            .route(
                "/importTable",
                web::post().to(
                    |gen: web::Data<GenController>, query: web::Query<TablesQuery>| async move {
                        gen.import_table(query).await
                    },
                ),
            )
            .route(
                "/createTable",
                web::post().to(
                    |gen: web::Data<GenController>, table: web::Json<GenTableModel>| async move {
                        gen.create_table(table).await
                    },
                ),
            )
            .route(
                "",
                web::put().to(
                    |gen: web::Data<GenController>, table: web::Json<UpdateGenTableRequest>| async move {
                        gen.update(table).await
                    },
                ),
            )
            .route(
                "/{tableIds}",
                web::delete().to(
                    |gen: web::Data<GenController>, path: web::Path<String>| async move {
                        gen.delete(path).await
                    },
                ),
            )
            .route(
                "/preview/{tableId}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.preview(path).await
                    },
                ),
            )
            .route(
                "/download/{tableName}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.download(path).await
                    },
                ),
            )
            .route(
                "/genCode/{tableName}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.gen_code(path).await
                    },
                ),
            )
            .route(
                "/synchDb/{tableName}",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.synch_db(path).await
                    },
                ),
            )
            .route(
                "/batchGenCode",
                web::get().to(
                    |gen: web::Data<GenController>, path: web::Path<i64>| async move {
                        gen.batch_gen_code(path).await
                    },
                ),
            ),
    );
}
