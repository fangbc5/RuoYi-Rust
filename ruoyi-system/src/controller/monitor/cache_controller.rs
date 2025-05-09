use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use lazy_static::lazy_static;
use log::info;
use ruoyi_common::utils::string::{redis_command_stats_to_map, redis_info_to_map};
use ruoyi_common::vo::{RData, RList};
use ruoyi_common::{constants, vo::R};
use ruoyi_framework::cache::get_global_cache;
use ruoyi_framework::config::AppConfig;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheVO {
    pub cache_name: String,
    pub cache_key: String,
    pub cache_value: String,
    pub remark: String,
}

impl CacheVO {
    pub fn new(cache_name: &str, remark: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            cache_key: String::new(),
            cache_value: String::new(),
            remark: remark.to_string(),
        }
    }
}

lazy_static! {
    static ref CACHE_NAME: Vec<CacheVO> = vec![
        CacheVO::new(constants::cache::USER_INFO_KEY, "用户信息"),
        CacheVO::new(constants::cache::SYS_CONFIG_KEY, "配置信息"),
        CacheVO::new(constants::cache::SYS_DICT_KEY, "字典信息"),
        CacheVO::new(constants::cache::CAPTCHA_KEY, "验证码"),
        CacheVO::new(constants::cache::REPEAT_SUBMIT_KEY, "防重提交"),
        CacheVO::new(constants::cache::RATE_LIMIT_KEY, "限流处理"),
        CacheVO::new(constants::cache::PWD_ERR_CNT_KEY, "密码错误次数"),
    ];
}

fn get_cache_vo(cache_name: &str) -> CacheVO {
    let cache_vo = CACHE_NAME.iter().find(|c| c.cache_name == cache_name);
    if let Some(cache_vo) = cache_vo {
        cache_vo.clone()
    } else {
        CacheVO::new(cache_name, "")
    }
}

#[get("/getNames")]
pub async fn get_names() -> impl Responder {
    HttpResponse::Ok().json(RList::ok_with_data(CACHE_NAME.clone()))
}

#[get("/getKeys/{cache_name}")]
pub async fn get_keys(path: web::Path<String>) -> impl Responder {
    // 从全局缓存中获取
    let cache_name = path.into_inner();
    match get_global_cache() {
        Ok(cache) => {
            if let Ok(keys) = cache.hkeys(&cache_name).await {
                HttpResponse::Ok().json(RList::ok_with_data(keys))
            } else {
                HttpResponse::Ok().json(RList::<String>::ok_with_data(vec![]))
            }
        }
        Err(_) => HttpResponse::Ok().json(RList::<String>::ok_with_data(vec![])),
    }
}

#[get("/getValue/{cache_name}/{cache_key}")]
pub async fn get_value(path: web::Path<(String, String)>) -> impl Responder {
    let (cache_name, cache_key) = path.into_inner();
    info!(
        "获取缓存值，参数: cache_name: {}, cache_key: {}",
        cache_name, cache_key
    );

    if let Ok(cache) = get_global_cache() {
        match cache_name.as_str() {
            constants::cache::USER_INFO_KEY
            | constants::cache::SYS_CONFIG_KEY
            | constants::cache::SYS_DICT_KEY => {
                if let Ok(value) = cache.hget_string(&cache_name, &cache_key).await {
                    if let Some(value) = value {
                        let mut cache_vo = get_cache_vo(&cache_name);
                        cache_vo.cache_key = cache_key;
                        cache_vo.cache_value = value;
                        return HttpResponse::Ok().json(RData::<CacheVO>::ok(cache_vo));
                    }
                }
            }
            _ => {
                return HttpResponse::Ok().json(R::<String>::fail("无法识别的缓存类型"));
            }
        }
    }
    HttpResponse::Ok().json(R::<String>::fail("获取缓存值失败"))
}

#[get("")]
pub async fn get_redis_info(config: web::Data<Arc<AppConfig>>) -> impl Responder {
    // 获取Redis信息
    use ruoyi_framework::cache::get_global_cache;
    use ruoyi_framework::config::cache::CacheType;

    match config.cache.cache_type {
        CacheType::Redis | CacheType::Multi => {
            if let Some(_redis_url) = &config.cache.redis.url {
                if let Ok(cache) = get_global_cache() {
                    // 获取Redis信息
                    let info = match cache.info(None).await {
                        Ok(info) => info,
                        _ => "".to_string(),
                    };
                    let info_map = redis_info_to_map(&info);
                    // 获取数据库大小
                    let db_size = match cache.dbsize().await {
                        Ok(db_size) => db_size,
                        _ => 0,
                    };

                    // 获取命令统计
                    let command_stats = match cache.info(Some("commandstats".to_string())).await {
                        Ok(command_stats) => command_stats,
                        _ => "".to_string(),
                    };
                    let command_stats_map = redis_command_stats_to_map(&command_stats);
                    HttpResponse::Ok().json(RData::<serde_json::Value>::ok(serde_json::json!({
                        "info": info_map,
                        "dbSize": db_size,
                        "commandStats": command_stats_map
                    })))
                } else {
                    HttpResponse::Ok().json(R::<String>::fail("无法获取全局缓存实例"))
                }
            } else {
                HttpResponse::Ok().json(R::<String>::fail("Redis配置不存在"))
            }
        }
        CacheType::Local => {
            HttpResponse::Ok().json(R::<String>::fail("当前使用的是本地缓存，无法获取Redis信息"))
        }
    }
}

pub fn load_cache_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cache")
            .service(get_names)
            .service(get_keys)
            .service(get_value)
            .service(get_redis_info),
    );
}
