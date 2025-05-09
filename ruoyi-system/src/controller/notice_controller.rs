// ruoyi-system/src/controller/notice_controller.rs
//! 通知公告控制器

use crate::service::notice_service::{
    CreateOrUpdateNoticeRequest, NoticeQuery, NoticeService, NoticeServiceImpl,
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use log::{error, info};
use ruoyi_common::{
    utils::string::option_is_empty,
    vo::{PageParam, RData, R},
};

/// 获取通知公告列表
#[get("/list")]
pub async fn get_notice_list(
    query: web::Query<NoticeQuery>,
    page_param: web::Query<PageParam>,
    notice_service: web::Data<NoticeServiceImpl>,
) -> impl Responder {
    info!("查询通知公告列表: {:?}", query);

    match notice_service.get_notice_list(query.0, page_param.0).await {
        Ok((notices, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": notices,
            "total": total
        }))),
        Err(e) => {
            error!("查询通知公告列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询通知公告列表失败: {}", e)))
        }
    }
}

/// 获取通知公告详情
#[get("/{id}")]
pub async fn get_notice(
    path: web::Path<i32>,
    notice_service: web::Data<NoticeServiceImpl>,
) -> impl Responder {
    let notice_id = path.into_inner();
    info!("查询通知公告: {}", notice_id);

    match notice_service.get_notice_by_id(notice_id).await {
        Ok(Some(notice)) => HttpResponse::Ok().json(RData::ok(notice)),
        Ok(None) => {
            HttpResponse::Ok().json(R::<String>::fail(&format!("通知公告不存在: {}", notice_id)))
        }
        Err(e) => {
            error!("查询通知公告失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询通知公告失败: {}", e)))
        }
    }
}

/// 新增通知公告
#[post("")]
pub async fn create_notice(
    req: web::Json<CreateOrUpdateNoticeRequest>,
    notice_service: web::Data<NoticeServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_notice_valid("create", &req.0).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("新增通知公告: {:?}", req);

    match notice_service.create_notice(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("新增通知公告成功")),
        Err(e) => {
            error!("新增通知公告失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("新增通知公告失败: {}", e)))
        }
    }
}

/// 修改通知公告
#[put("")]
pub async fn update_notice(
    req: web::Json<CreateOrUpdateNoticeRequest>,
    notice_service: web::Data<NoticeServiceImpl>,
) -> impl Responder {
    info!("修改通知公告: {:?}", req);

    // 校验必填字段
    if req.notice_id.is_none() {
        return HttpResponse::Ok().json(R::<String>::fail("公告ID不能为空"));
    }

    match notice_service.update_notice(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("修改通知公告成功")),
        Err(e) => {
            error!("修改通知公告失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("修改通知公告失败: {}", e)))
        }
    }
}

/// 删除通知公告
#[delete("/{ids}")]
pub async fn delete_notices(
    path: web::Path<String>,
    notice_service: web::Data<NoticeServiceImpl>,
) -> impl Responder {
    let ids = path.into_inner();
    info!("删除通知公告: {}", ids);

    // 解析ID列表
    let notice_ids = ids
        .split(',')
        .filter_map(|id| id.parse::<i32>().ok())
        .collect::<Vec<i32>>();

    if notice_ids.is_empty() {
        return HttpResponse::Ok().json(R::<String>::fail("公告ID格式不正确"));
    }

    match notice_service.delete_notices(notice_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除通知公告成功")),
        Err(e) => {
            error!("删除通知公告失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除通知公告失败: {}", e)))
        }
    }
}

async fn check_notice_valid(
    action: &str,
    req: &CreateOrUpdateNoticeRequest,
) -> (bool, Option<String>) {
    if action == "update" {
        if req.notice_id.is_none() {
            return (false, Some("公告ID不能为空".to_string()));
        }
    }
    // 校验必填字段
    if option_is_empty(&req.notice_title) {
        return (false, Some("公告标题不能为空".to_string()));
    }
    if option_is_empty(&req.notice_type) {
        return (false, Some("公告类型不能为空".to_string()));
    }
    if option_is_empty(&req.status) {
        return (false, Some("公告状态不能为空".to_string()));
    }
    if option_is_empty(&req.notice_content) {
        return (false, Some("公告内容不能为空".to_string()));
    }

    (true, None)
}
/// 加载通知公告路由
pub fn load_notice_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notice")
            .service(get_notice_list)
            .service(get_notice)
            .service(create_notice)
            .service(update_notice)
            .service(delete_notices),
    );
}
