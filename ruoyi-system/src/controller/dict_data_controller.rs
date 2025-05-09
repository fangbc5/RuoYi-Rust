/// 字典数据控制器
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use log::{error, info};
use ruoyi_common::{utils::string::option_is_empty, vo::{PageParam, RData, RList, R}};
use serde::Deserialize;

use crate::{
    entity::prelude::*,
    service::dict_data_service::{DictDataService, DictDataServiceImpl},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictDataQuery {
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateDictDataRequest {
    pub dict_code: Option<i64>,
    pub dict_label: Option<String>,
    pub dict_value: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<String>,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
    pub is_default: Option<String>,
    pub remark: Option<String>,
    pub dict_sort: Option<i32>,
}

#[get("/list")]
pub async fn get_dict_data_list(
    query: web::Query<DictDataQuery>,
    page_param: web::Query<PageParam>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    info!("获取字典数据列表: {:?}", query);

    match dict_data_service
        .get_dict_data_list(query.0, page_param.0)
        .await
    {
        Ok((dict_datas, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": dict_datas,
            "total": total,
        }))),
        Err(e) => {
            error!("获取字典数据失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取字典数据失败: {}", e)))
        }
    }
}

#[get("/{dict_code}")]
pub async fn get_dict_data(
    path: web::Path<i64>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    let dict_id = path.into_inner();
    info!("获取字典数据: {}", dict_id);
    match dict_data_service.get_dict_data_by_id(dict_id).await {
        Ok(dict_data) => HttpResponse::Ok().json(RData::ok(dict_data)),
        Err(e) => {
            error!("获取字典数据失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取字典数据失败: {}", e)))
        }
    }
}

#[post("")]
pub async fn create_dict_data(
    req: web::Json<CreateOrUpdateDictDataRequest>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_dict_data_valid("create", &req.0, &dict_data_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("创建字典数据: {:?}", req);
    match dict_data_service.create_dict_data(req.0).await {
        Ok(dict_data) => HttpResponse::Ok().json(RData::ok(dict_data)),
        Err(e) => {
            error!("创建字典数据失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建字典数据失败: {}", e)))
        }
    }
}

#[put("")]
pub async fn update_dict_data(
    req: web::Json<CreateOrUpdateDictDataRequest>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_dict_data_valid("update", &req.0, &dict_data_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("更新字典数据: {:?}", req);
    match dict_data_service.update_dict_data(req.0).await {
        Ok(dict_data) => HttpResponse::Ok().json(RData::ok(dict_data)),
        Err(e) => {
            error!("更新字典数据失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新字典数据失败: {}", e)))
        }
    }
}

#[delete("/{ids}")]
pub async fn delete_dict_data(
    path: web::Path<String>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    let ids = path.into_inner();
    info!("删除字典数据: {}", ids);
    let ids = ids
        .split(',')
        .map(|id| id.parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    match dict_data_service.delete_dict_datas(ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除字典数据成功")),
        Err(e) => {
            error!("删除字典数据失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除字典数据失败: {}", e)))
        }
    }
}

#[get("/type/{dictType}")]
pub async fn dict_type(
    path: web::Path<String>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    let dict_type = path.into_inner();
    match dict_data_service.get_dict_data_by_type(&dict_type).await {
        Ok(dict_data) => HttpResponse::Ok().json(RList::<DictDataModel>::ok_with_data(dict_data)),
        Err(e) => {
            error!("获取字典数据失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("获取字典数据失败: {}", e)))
        }
    }
}

async fn check_dict_data_valid(
    action: &str,
    req: &CreateOrUpdateDictDataRequest,
    dict_data_service: &DictDataServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" {
        if req.dict_code.is_none() {
            return (false, Some("字典数据id不能为空".to_string()));
        }
    }
    // 字典类型
    if option_is_empty(&req.dict_type) {
        return (false, Some("字典类型不能为空".to_string()));
    }
    // 字典标签
    if option_is_empty(&req.dict_label) {
        return (false, Some("字典标签不能为空".to_string()));
    }
    // 字典键值
    if option_is_empty(&req.dict_value) {
        return (false, Some("字典键值不能为空".to_string()));
    }
    // 字典排序
    if req.dict_sort.is_none() {
        return (false, Some("字典排序不能为空".to_string()));
    }
    // 字典状态
    if option_is_empty(&req.status) {
        return (false, Some("字典状态不能为空".to_string()));
    }
    // 字典标签是否唯一
    match dict_data_service
        .check_dict_data_label_unique(
            req.dict_type.as_ref().unwrap(),
            req.dict_label.as_ref().unwrap(),
            req.dict_code,
        )
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("字典标签已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查字典标签是否存在失败: {}", e);
            return (false, Some("检查字典标签是否存在失败".to_string()));
        }
    }

    (true, None)
}
/// 路由注册
pub fn load_dict_data_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dict/data")
            .service(get_dict_data_list)
            .service(dict_type)
            .service(get_dict_data)
            .service(create_dict_data)
            .service(update_dict_data)
            .service(delete_dict_data),
    );
}
