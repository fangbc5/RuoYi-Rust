use crate::service::dict_data_service::{DictDataService, DictDataServiceImpl};
use crate::service::dict_type_service::{DictTypeService, DictTypeServiceImpl};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use ruoyi_common::constants;
use ruoyi_common::utils::string::option_is_empty;
use ruoyi_common::utils::time::deserialize_optional_datetime;
use ruoyi_common::vo::{PageParam, RData, RList, R};
use ruoyi_framework::cache::get_global_cache;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictTypeQuery {
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<String>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateDictTypeRequest {
    pub dict_id: Option<i64>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<String>,
    pub remark: Option<String>,
}

#[get("/list")]
pub async fn get_dict_type_list(
    query: web::Query<DictTypeQuery>,
    page_param: web::Query<PageParam>,
    dict_type_service: web::Data<DictTypeServiceImpl>,
) -> impl Responder {
    info!("查询字典类型列表: {:?}", query);
    match dict_type_service
        .get_dict_type_list(query.0, page_param.0)
        .await
    {
        Ok((dict_types, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": dict_types,
            "total": total
        }))),
        Err(e) => {
            error!("查询字典类型列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询字典类型列表失败: {}", e)))
        }
    }
}

#[get("/{id}")]
pub async fn get_dict_type(
    path: web::Path<i64>,
    dict_type_service: web::Data<DictTypeServiceImpl>,
) -> impl Responder {
    let dict_id = path.into_inner();
    info!("查询字典类型: {:?}", dict_id);
    match dict_type_service.get_dict_type(dict_id).await {
        Ok(dict_type) => HttpResponse::Ok().json(RData::ok(dict_type)),
        Err(e) => {
            error!("查询字典类型失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询字典类型失败: {}", e)))
        }
    }
}

#[post("")]
pub async fn create_dict_type(
    req: web::Json<CreateOrUpdateDictTypeRequest>,
    dict_type_service: web::Data<DictTypeServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_dict_type_valid("create", &req.0, &dict_type_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("创建字典类型: {:?}", req);
    match dict_type_service.create_dict_type(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建字典类型成功")),
        Err(e) => {
            error!("创建字典类型失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建字典类型失败: {}", e)))
        }
    }
}

#[put("")]
pub async fn update_dict_type(
    req: web::Json<CreateOrUpdateDictTypeRequest>,
    dict_type_service: web::Data<DictTypeServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_dict_type_valid("update", &req.0, &dict_type_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("更新字典类型: {:?}", req);
    match dict_type_service.update_dict_type(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("更新字典类型成功")),
        Err(e) => {
            error!("更新字典类型失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新字典类型失败: {}", e)))
        }
    }
}

#[delete("/{ids}")]
pub async fn delete_dict_types(
    path: web::Path<String>,
    dict_type_service: web::Data<DictTypeServiceImpl>,
) -> impl Responder {
    info!("删除字典类型: {:?}", path);
    let ids = path.into_inner();
    let ids = ids
        .split(',')
        .map(|id| id.parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    match dict_type_service.delete_dict_types(ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除字典类型成功")),
        Err(e) => {
            error!("删除字典类型失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除字典类型失败: {}", e)))
        }
    }
}

#[get("/optionselect")]
pub async fn option_select(dict_type_service: web::Data<DictTypeServiceImpl>) -> impl Responder {
    match dict_type_service.get_all_dict_types().await {
        Ok(dict_types) => HttpResponse::Ok().json(RList::ok_with_data(dict_types)),
        Err(e) => {
            error!("查询字典类型失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询字典类型失败: {}", e)))
        }
    }
}

#[delete("/refreshCache")]
pub async fn refresh_cache(
    dict_type_service: web::Data<DictTypeServiceImpl>,
    dict_data_service: web::Data<DictDataServiceImpl>,
) -> impl Responder {
    info!("刷新字典类型缓存");
    if let Ok(cache) = get_global_cache() {
        match dict_type_service.get_all_dict_types().await {
            Ok(dict_types) => {
                // 遍历所有的字典类型并查询字典数据
                for dict_type in dict_types {
                    if let Some(dict_type) = dict_type.dict_type {
                        if let Ok(dict_data) =
                            dict_data_service.get_dict_data_by_type(&dict_type).await
                        {
                            // 将字典数据缓存到redis
                            let _ = cache
                                .hset_string(
                                    constants::cache::SYS_DICT_KEY,
                                    &dict_type,
                                    &serde_json::to_string(&dict_data).unwrap(),
                                )
                                .await;
                        }
                    }
                }
                return HttpResponse::Ok().json(R::<String>::ok_with_msg("刷新缓存成功"));
            }
            Err(e) => {
                error!("刷新缓存失败: {}", e);
                return HttpResponse::Ok().json(R::<String>::fail(&format!("刷新缓存失败: {}", e)));
            }
        };
    }
    HttpResponse::Ok().json(R::<String>::fail(&format!("刷新缓存失败")))
}

async fn check_dict_type_valid(
    action: &str,
    req: &CreateOrUpdateDictTypeRequest,
    dict_type_service: &DictTypeServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" {
        if req.dict_id.is_none() {
            return (false, Some("字典类型ID不能为空".to_string()));
        }
    }
    if option_is_empty(&req.dict_type) {
        return (false, Some("字典类型不能为空".to_string()));
    }
    if option_is_empty(&req.dict_name) {
        return (false, Some("字典名称不能为空".to_string()));
    }
    if option_is_empty(&req.status) {
        return (false, Some("字典状态不能为空".to_string()));
    }
    match dict_type_service
        .check_dict_type_unique(req.dict_type.as_ref().unwrap(), req.dict_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("字典类型已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查字典类型是否存在失败: {}", e);
            return (false, Some("检查字典类型是否存在失败".to_string()));
        }
    }
    match dict_type_service
        .check_dict_type_name_unique(req.dict_name.as_ref().unwrap(), req.dict_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("字典名称已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查字典名称是否存在失败: {}", e);
            return (false, Some("检查字典名称是否存在失败".to_string()));
        }
    }
    (true, None)
}

pub fn load_dict_type_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dict/type")
            .service(get_dict_type_list)
            .service(option_select)
            .service(get_dict_type)
            .service(create_dict_type)
            .service(update_dict_type)
            .service(refresh_cache)
            .service(delete_dict_types),
    );
}
