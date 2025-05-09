// ruoyi-system/src/controller/login_info_controller.rs
//! 登录日志控制器

use crate::service::login_info_service::{
    CreateLoginInfoRequest, LoginInfoQuery, LoginInfoService, LoginInfoServiceImpl,
};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use log::{error, info};
use ruoyi_common::vo::{PageParam, RData, R};

/// 获取登录日志列表
#[get("/list")]
pub async fn get_login_info_list(
    query: web::Query<LoginInfoQuery>,
    page_param: web::Query<PageParam>,
    login_info_service: web::Data<LoginInfoServiceImpl>,
) -> impl Responder {
    info!("查询登录日志列表: {:?}", query);

    match login_info_service
        .get_login_info_list(query.0, page_param.0)
        .await
    {
        Ok((login_infos, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": login_infos,
            "total": total
        }))),
        Err(e) => {
            error!("查询登录日志列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询登录日志列表失败: {}", e)))
        }
    }
}

/// 获取登录日志详情
#[get("/{id}")]
pub async fn get_login_info(
    path: web::Path<i64>,
    login_info_service: web::Data<LoginInfoServiceImpl>,
) -> impl Responder {
    let info_id = path.into_inner();
    info!("查询登录日志: {}", info_id);

    match login_info_service.get_login_info_by_id(info_id).await {
        Ok(Some(login_info)) => HttpResponse::Ok().json(RData::ok(login_info)),
        Ok(None) => {
            HttpResponse::Ok().json(R::<String>::fail(&format!("登录日志不存在: {}", info_id)))
        }
        Err(e) => {
            error!("查询登录日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询登录日志失败: {}", e)))
        }
    }
}

/// 记录登录信息
#[post("")]
pub async fn record_login_info(
    req: web::Json<CreateLoginInfoRequest>,
    login_info_service: web::Data<LoginInfoServiceImpl>,
) -> impl Responder {
    info!("记录登录信息: {:?}", req);

    match login_info_service.record_login_info(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("记录登录信息成功")),
        Err(e) => {
            error!("记录登录信息失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("记录登录信息失败: {}", e)))
        }
    }
}

/// 删除登录日志
#[delete("/{ids}")]
pub async fn delete_login_infos(
    path: web::Path<String>,
    login_info_service: web::Data<LoginInfoServiceImpl>,
) -> impl Responder {
    let ids = path.into_inner();
    info!("删除登录日志: {}", ids);

    // 解析ID列表
    let login_info_ids = ids
        .split(',')
        .filter_map(|id| id.parse::<i64>().ok())
        .collect::<Vec<i64>>();

    if login_info_ids.is_empty() {
        return HttpResponse::Ok().json(R::<String>::fail("登录日志ID格式不正确"));
    }

    match login_info_service.delete_login_infos(login_info_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除登录日志成功")),
        Err(e) => {
            error!("删除登录日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除登录日志失败: {}", e)))
        }
    }
}

/// 清空登录日志
#[delete("/clean")]
pub async fn clean_login_info(
    login_info_service: web::Data<LoginInfoServiceImpl>,
) -> impl Responder {
    info!("清空登录日志");

    match login_info_service.clean_login_info().await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("清空登录日志成功")),
        Err(e) => {
            error!("清空登录日志失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("清空登录日志失败: {}", e)))
        }
    }
}

/// 加载登录日志路由
pub fn load_login_info_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/logininfor")
            .service(get_login_info_list)
            .service(get_login_info)
            .service(record_login_info)
            .service(clean_login_info)
            .service(delete_login_infos),
    );
}
