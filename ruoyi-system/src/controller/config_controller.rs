use crate::service::config_service::{ConfigService, ConfigServiceImpl};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use ruoyi_common::constants;
use ruoyi_common::utils::string::option_is_empty;
use ruoyi_common::utils::time::deserialize_optional_datetime;
use ruoyi_common::vo::{PageParam, RData, R};
use ruoyi_framework::cache::get_global_cache;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigQuery {
    /// 参数键名
    pub config_key: Option<String>,
    /// 参数名称
    pub config_name: Option<String>,
    /// 参数类型
    pub config_type: Option<String>,

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
pub struct CreateOrUpdateConfigRequest {
    pub config_id: Option<i32>,
    pub config_key: Option<String>,
    pub config_name: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
}

#[get("/list")]
pub async fn get_config_list(
    query: web::Query<ConfigQuery>,
    page_param: web::Query<PageParam>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    info!("获取配置列表: {:?}", query);

    match config_service.get_config_list(query.0, page_param.0).await {
        Ok((configs, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "rows": configs,
            "total": total,
        }))),
        Err(e) => {
            error!("获取配置列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取配置列表失败: {}", e)))
        }
    }
}

#[get("/{configId}")]
pub async fn get_config(
    path: web::Path<i32>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    let config_id = path.into_inner();
    info!("获取配置: {:?}", config_id);
    match config_service.get_config_by_id(config_id).await {
        Ok(config) => HttpResponse::Ok().json(RData::ok(config)),
        Err(e) => {
            error!("获取配置失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取配置失败: {}", e)))
        }
    }
}

#[post("")]
pub async fn create_config(
    req: web::Json<CreateOrUpdateConfigRequest>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    let (valid, err_msg) = check_config_valid("create", &req.0, &config_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&err_msg.unwrap()));
    }
    info!("创建配置: {:?}", req);
    match config_service.create_config(req.0).await {
        Ok(config) => HttpResponse::Ok().json(RData::ok(config)),
        Err(e) => {
            error!("创建配置失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建配置失败: {}", e)))
        }
    }
}

#[put("")]
pub async fn update_config(
    req: web::Json<CreateOrUpdateConfigRequest>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    let (valid, err_msg) = check_config_valid("update", &req.0, &config_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&err_msg.unwrap()));
    }
    info!("更新配置: {:?}", req);
    match config_service.update_config(req.0).await {
        Ok(config) => HttpResponse::Ok().json(RData::ok(config)),
        Err(e) => {
            error!("更新配置失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新配置失败: {}", e)))
        }
    }
}

#[delete("/{configIds}")]
pub async fn delete_configs(
    path: web::Path<String>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    let config_ids = path.into_inner();
    let config_ids = config_ids
        .split(',')
        .map(|id| id.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    info!("删除配置: {:?}", config_ids);
    match config_service.delete_configs(config_ids).await {
        Ok(num) => {
            HttpResponse::Ok().json(R::<String>::ok_with_msg(&format!("删除配置成功: {}", num)))
        }
        Err(e) => {
            error!("删除配置失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除配置失败: {}", e)))
        }
    }
}

async fn check_config_valid(
    actoin: &str,
    req: &CreateOrUpdateConfigRequest,
    config_service: &ConfigServiceImpl,
) -> (bool, Option<String>) {
    if actoin == "update" {
        if req.config_id.is_none() {
            return (false, Some("配置ID不能为空".to_string()));
        }
    }
    if option_is_empty(&req.config_key) {
        return (false, Some("配置键不能为空".to_string()));
    }
    if option_is_empty(&req.config_name) {
        return (false, Some("配置名称不能为空".to_string()));
    }
    if option_is_empty(&req.config_value) {
        return (false, Some("配置值不能为空".to_string()));
    }
    if option_is_empty(&req.config_type) {
        return (false, Some("配置类型不能为空".to_string()));
    }
    // 参数名称唯一性校验
    match config_service
        .check_config_name_unique(&req.config_name.as_ref().unwrap(), req.config_id)
        .await
    {
        Ok(valid) => {
            if !valid {
                return (false, Some("配置名称已存在".to_string()));
            }
        }
        Err(e) => {
            error!("配置名称唯一性校验失败: {}", e);
            return (false, Some("配置名称唯一性校验失败".to_string()));
        }
    }
    // 参数键名唯一性校验
    match config_service
        .check_config_key_unique(&req.config_key.as_ref().unwrap(), req.config_id)
        .await
    {
        Ok(valid) => {
            if !valid {
                return (false, Some("配置键名已存在".to_string()));
            }
        }
        Err(e) => {
            error!("配置键名唯一性校验失败: {}", e);
            return (false, Some("配置键名唯一性校验失败".to_string()));
        }
    }

    (true, None)
}
#[get("/configKey/{configKey}")]
pub async fn get_config_key(
    path: web::Path<String>,
    config_service: web::Data<ConfigServiceImpl>,
) -> impl Responder {
    let config_key = path.into_inner();
    match config_service.get_config_by_key(&config_key).await {
        Ok(val) => HttpResponse::Ok().json(R::<String>::ok_with_msg(&val)),
        Err(e) => {
            error!("获取配置失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("获取配置失败: {}", e)))
        }
    }
}

#[delete("/refreshCache")]
pub async fn refresh_cache(config_service: web::Data<ConfigServiceImpl>) -> impl Responder {
    info!("刷新配置缓存");
    if let Ok(cache) = get_global_cache() {
        match config_service.get_all_configs().await {
            Ok(configs) => {
                for config in configs {
                    let _ = cache
                        .set_string(
                            &format!(
                                "{}{}",
                                constants::cache::SYS_CONFIG_PREFIX,
                                config.config_key.unwrap()
                            ),
                            &config.config_value.unwrap_or_default(),
                        )
                        .await;
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

/// 注册配置控制器路由
pub fn load_config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .service(get_config_list)
            .service(get_config_key)
            .service(get_config)
            .service(create_config)
            .service(update_config)
            .service(refresh_cache)
            .service(delete_configs),
    );
}
