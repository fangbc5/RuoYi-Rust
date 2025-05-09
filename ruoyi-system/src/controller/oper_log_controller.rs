// ruoyi-system/src/controller/oper_log_controller.rs
//! 操作日志控制器

use crate::service::oper_log_service::{
    CreateOperLogRequest, OperLogQuery, OperLogService, OperLogServiceImpl,
};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use log::{error, info};
use ruoyi_common::vo::{PageParam, RData, R};

/// 获取操作日志列表
#[get("/list")]
pub async fn get_oper_log_list(
    query: web::Query<OperLogQuery>,
    page_param: web::Query<PageParam>,
    oper_log_service: web::Data<OperLogServiceImpl>,
) -> impl Responder {
    info!("查询操作日志列表: {:?}", query);

    match oper_log_service
        .get_oper_log_list(query.0, page_param.0)
        .await
    {
        Ok((oper_logs, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": oper_logs,
            "total": total
        }))),
        Err(e) => {
            error!("查询操作日志列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询操作日志列表失败: {}", e)))
        }
    }
}

/// 获取操作日志详情
#[get("/{id}")]
pub async fn get_oper_log(
    path: web::Path<i64>,
    oper_log_service: web::Data<OperLogServiceImpl>,
) -> impl Responder {
    let oper_id = path.into_inner();
    info!("查询操作日志: {}", oper_id);

    match oper_log_service.get_oper_log_by_id(oper_id).await {
        Ok(Some(oper_log)) => HttpResponse::Ok().json(RData::ok(oper_log)),
        Ok(None) => {
            HttpResponse::Ok().json(R::<String>::fail(&format!("操作日志不存在: {}", oper_id)))
        }
        Err(e) => {
            error!("查询操作日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询操作日志失败: {}", e)))
        }
    }
}

/// 记录操作日志
#[post("")]
pub async fn record_oper_log(
    req: web::Json<CreateOperLogRequest>,
    oper_log_service: web::Data<OperLogServiceImpl>,
) -> impl Responder {
    info!("记录操作日志: {:?}", req);

    match oper_log_service.record_oper_log(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("记录操作日志成功")),
        Err(e) => {
            error!("记录操作日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("记录操作日志失败: {}", e)))
        }
    }
}

/// 删除操作日志
#[delete("/{ids}")]
pub async fn delete_oper_logs(
    path: web::Path<String>,
    oper_log_service: web::Data<OperLogServiceImpl>,
) -> impl Responder {
    let ids = path.into_inner();
    info!("删除操作日志: {}", ids);

    // 解析ID列表
    let oper_log_ids = ids
        .split(',')
        .filter_map(|id| id.parse::<i64>().ok())
        .collect::<Vec<i64>>();

    if oper_log_ids.is_empty() {
        return HttpResponse::Ok().json(R::<String>::fail("操作日志ID格式不正确"));
    }

    match oper_log_service.delete_oper_logs(oper_log_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除操作日志成功")),
        Err(e) => {
            error!("删除操作日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除操作日志失败: {}", e)))
        }
    }
}

/// 清空操作日志
#[delete("/clean")]
pub async fn clean_oper_log(oper_log_service: web::Data<OperLogServiceImpl>) -> impl Responder {
    info!("清空操作日志");

    match oper_log_service.clean_oper_log().await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("清空操作日志成功")),
        Err(e) => {
            error!("清空操作日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("清空操作日志失败: {}", e)))
        }
    }
}

/// 加载操作日志路由
pub fn load_oper_log_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/operlog")
            .service(get_oper_log_list)
            .service(get_oper_log)
            .service(record_oper_log)
            .service(clean_oper_log)
            .service(delete_oper_logs),
    );
}
