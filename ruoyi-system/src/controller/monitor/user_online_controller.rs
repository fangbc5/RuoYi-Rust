use crate::entity::vo::user::UserOnline;
use actix_web::{delete, get, Responder};
use actix_web::{web, HttpResponse};
use log::error;
use ruoyi_common::constants;
use ruoyi_common::utils::string::option_is_empty;
use ruoyi_common::vo::R;
use ruoyi_framework::cache::get_global_cache;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOnlineQueryParams {
    pub user_name: Option<String>,
    pub ipaddr: Option<String>,
}

#[get("/list")]
pub async fn get_user_online_list(params: web::Query<UserOnlineQueryParams>) -> impl Responder {
    if let Ok(cache) = get_global_cache() {
        // 获取所有在线用户的token列表
        let mut online_users = Vec::new();

        if let Ok(keys) = cache
            .keys(&format!("{}*", constants::cache::TOKEN_PREFIX))
            .await
        {
            for key in keys {
                if let Ok(Some(user)) = cache.get_string(&key).await {
                    match serde_json::from_str::<UserOnline>(&user) {
                        Ok(user_info) => {
                            online_users.push(user_info);
                        }
                        Err(e) => {
                            error!("解析用户在线信息失败: {}", e);
                        }
                    }
                }
            }
        }

        // 获取总数
        let total = online_users.len();

        // 根据查询条件过滤
        if !option_is_empty(&params.user_name) || !option_is_empty(&params.ipaddr) {
            online_users = online_users
                .into_iter()
                .filter(|user| {
                    if !option_is_empty(&params.user_name) && !option_is_empty(&user.user_name) {
                        if user
                            .user_name
                            .as_ref()
                            .unwrap()
                            .contains(params.user_name.as_ref().unwrap())
                        {
                            return true;
                        }
                    }

                    if !option_is_empty(&params.ipaddr) && !option_is_empty(&user.ipaddr) {
                        if user
                            .ipaddr
                            .as_ref()
                            .unwrap()
                            .contains(params.ipaddr.as_ref().unwrap())
                        {
                            return true;
                        }
                    }
                    false
                })
                .collect::<Vec<_>>();
        }

        return HttpResponse::Ok().json(R::<serde_json::Value>::ok_with_data(serde_json::json!({
            "rows": online_users,
            "total": total,
        })));
    }
    HttpResponse::Ok().json(R::<serde_json::Value>::ok_with_data(serde_json::json!({
        "rows": Vec::<UserOnline>::new(),
        "total": 0,
    })))
}

#[delete("/{tokenId}")]
pub async fn force_logout(path: web::Path<String>) -> impl Responder {
    let token_id = path.into_inner();
    if let Ok(cache) = get_global_cache() {
        let _ = cache
            .del(&format!("{}{}", constants::cache::TOKEN_PREFIX, token_id))
            .await;
        HttpResponse::Ok().json(R::<String>::ok_with_msg("强制退出成功"))
    } else {
        HttpResponse::Ok().json(R::<String>::fail("强制退出失败"))
    }
}
pub fn load_user_online_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/online")
            .service(get_user_online_list)
            .service(force_logout),
    );
}
