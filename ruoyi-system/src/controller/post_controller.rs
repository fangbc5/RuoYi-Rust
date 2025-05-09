use crate::service::post_service::{PostService, PostServiceImpl};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use log::{error, info};
use ruoyi_common::{utils::string::option_is_empty, vo::{PageParam, RData, R}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostQuery {
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePostRequest {
    pub post_id: Option<i64>,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub post_sort: Option<i32>,
    pub status: Option<String>,
    pub remark: Option<String>,
}

#[get("/list")]
pub async fn get_post_list(
    req: web::Query<PostQuery>,
    page_param: web::Query<PageParam>,
    post_service: web::Data<PostServiceImpl>,
) -> impl Responder {
    info!("查询岗位列表: {:?}", req);

    match post_service.get_post_list(req.0, page_param.0).await {
        Ok((posts, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": posts,
            "total": total
        }))),
        Err(e) => {
            error!("查询岗位列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询岗位列表失败: {}", e)))
        }
    }
}

#[get("/{id}")]
pub async fn get_post(
    path: web::Path<i64>,
    post_service: web::Data<PostServiceImpl>,
) -> impl Responder {
    info!("查询岗位: {:?}", path);
    let post_id = path.into_inner();
    match post_service.get_post(post_id).await {
        Ok(post) => HttpResponse::Ok().json(RData::ok(post)),
        Err(e) => {
            error!("查询岗位失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询岗位失败: {}", e)))
        }
    }
}

#[post("")]
pub async fn create_post(
    req: web::Json<CreateOrUpdatePostRequest>,
    post_service: web::Data<PostServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_post_valid("create", &req.0, &post_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("创建岗位: {:?}", req);
    match post_service.create_post(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建岗位成功")),
        Err(e) => {
            error!("创建岗位失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建岗位失败: {}", e)))
        }
    }
}

#[put("")]
pub async fn update_post(
    req: web::Json<CreateOrUpdatePostRequest>,
    post_service: web::Data<PostServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_post_valid("update", &req.0, &post_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("更新岗位: {:?}", req);
    match post_service.update_post(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("更新岗位成功")),
        Err(e) => {
            error!("更新岗位失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新岗位失败: {}", e)))
        }
    }
}

#[delete("/{ids}")]
pub async fn delete_posts(
    path: web::Path<String>,
    post_service: web::Data<PostServiceImpl>,
) -> impl Responder {
    let ids = path.into_inner();
    let ids = ids
        .split(',')
        .map(|id| id.parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    match post_service.delete_post(ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除岗位成功")),
        Err(e) => {
            error!("删除岗位失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除岗位失败: {}", e)))
        }
    }
}

async fn check_post_valid(
    action: &str,
    req: &CreateOrUpdatePostRequest,
    post_service: &PostServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" {
        if req.post_id.is_none() {
            return (false, Some("岗位ID不能为空".to_string()));
        }
    }

    if option_is_empty(&req.post_code) {
        return (false, Some("岗位编码不能为空".to_string()));
    }

    if option_is_empty(&req.post_name) {
        return (false, Some("岗位名称不能为空".to_string()));
    }
    if req.post_sort.is_none() {
        return (false, Some("岗位排序不能为空".to_string()));
    }
    if option_is_empty(&req.status) {
        return (false, Some("岗位状态不能为空".to_string()));
    }

    //岗位名称是否重复
    match post_service
        .check_post_name_unique(req.post_name.as_ref().unwrap(), req.post_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("岗位名称已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查岗位名称是否重复失败: {}", e);
            return (false, Some("检查岗位名称是否重复失败".to_string()));
        }
    }

    //岗位编码是否重复
    match post_service
        .check_post_code_unique(req.post_code.as_ref().unwrap(), req.post_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("岗位编码已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查岗位编码是否重复失败: {}", e);
            return (false, Some("检查岗位编码是否重复失败".to_string()));
        }
    }
    (true, None)
}

pub fn load_post_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/post")
            .service(get_post_list)
            .service(get_post)
            .service(create_post)
            .service(update_post)
            .service(delete_posts),
    );
}
