use std::sync::Arc;

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use log::{error, info};
use ruoyi_common::{
    constants,
    utils::{
        http, ip,
        jwt::{generate_token, Claims},
        password::verify_password,
    },
    vo::{RList, R},
};
use ruoyi_framework::{
    cache::get_global_cache,
    config::AppConfig,
    logger::entity::LoginInfoModel,
    web::{
        service::captcha::{CaptchaService, InMemoryCaptchaService},
        tls,
    },
};
use serde::Deserialize;

use crate::{
    entity::vo::user::UserInfo,
    service::{
        menu_service::{MenuService, MenuServiceImpl},
        user_service::{UserService, UserServiceImpl},
    },
};
/// 登录请求参数
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 验证码
    pub code: String,
    /// 验证码唯一标识
    pub uuid: String,
    /// 记住我
    #[serde(default)]
    pub remember_me: bool,
}

/// 用户登录
#[post("/login")]
pub async fn login(
    req: web::Json<LoginRequest>,
    request: HttpRequest,
    user_service: web::Data<UserServiceImpl>,
    captcha_service: web::Data<InMemoryCaptchaService>,
    config: web::Data<Arc<AppConfig>>,
) -> impl Responder {
    info!("用户登录请求: username={}", req.username);
    // 构建出登录日志结构体
    let ipaddr = ip::get_real_ip_by_request(&request);
    let login_location = ip::get_ip_location(&ipaddr);
    let mut login_info = LoginInfoModel {
        info_id: 0,
        user_name: Some(req.username.clone()),
        ipaddr: Some(ipaddr.clone()),
        login_location: Some(login_location.clone()),
        browser: Some(http::get_browser_info(&request)),
        os: Some(http::get_os_info(&request)),
        status: Some("".to_string()),
        msg: Some("".to_string()),
        login_time: Some(Utc::now()),
    };
    // 1. 验证验证码
    if !captcha_service.verify_captcha(&req.uuid, &req.code) {
        login_info.msg = Some("验证码错误".to_string());
        // 记录执行时间
        error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
        return HttpResponse::Ok().json(R::<String>::fail("验证码错误"));
    }

    // 2. 验证用户名和密码
    match user_service.get_user_by_username(&req.username).await {
        Ok(user) => {
            if let Some(user) = user {
                // 校验密码
                if !verify_password(
                    &req.password,
                    user.password.as_ref().unwrap_or(&"".to_string()),
                ) {
                    login_info.msg = Some("用户名或密码错误".to_string());
                    error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
                    return HttpResponse::Ok().json(R::<String>::fail("用户名或密码错误"));
                }
                // 实际生产中应当验证密码
                if user.is_active() {
                    // 生成JWT令牌
                    let token = match generate_token(
                        user.user_id as i64,
                        &user.user_name,
                        &config.jwt.secret,
                        config.jwt.expires_in,
                    ) {
                        Ok(token) => token,
                        Err(e) => {
                            login_info.msg = Some(format!("生成令牌失败: {}", e));
                            error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
                            return HttpResponse::Ok()
                                .json(R::<String>::fail(&format!("生成令牌失败: {}", e)));
                        }
                    };

                    // 返回令牌
                    login_info.msg = Some("登录成功".to_string());
                    info!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
                    // 缓存用户信息
                    if let Ok(cache) = get_global_cache() {
                        cache
                            .hset_string(
                                constants::cache::USER_INFO_KEY,
                                &user.user_name,
                                &serde_json::to_string(&user).unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                    HttpResponse::Ok().json(serde_json::json!({
                        "code": 200,
                        "msg": "登录成功",
                        "token": token
                    }))
                } else {
                    login_info.msg = Some("用户已禁用".to_string());
                    error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
                    HttpResponse::Ok().json(R::<String>::fail("用户已被禁用"))
                }
            } else {
                login_info.msg = Some("用户不存在".to_string());
                error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
                HttpResponse::Ok().json(R::<String>::fail("用户不存在"))
            }
        }
        Err(e) => {
            login_info.msg = Some(format!("获取用户信息失败: {}", e));
            error!(target: "system::login_info", "{}", serde_json::to_string(&login_info).unwrap());
            HttpResponse::Ok().json(R::<String>::fail("获取用户信息失败"))
        }
    }
}

/// 退出登录
#[post("/logout")]
pub async fn logout() -> impl Responder {
    // 实际上服务端JWT不需要主动失效，前端清除token即可
    if let Some(user_context) = tls::get_sync_user_context() {
        if let Ok(cache) = get_global_cache() {
            cache
                .hdel(constants::cache::USER_INFO_KEY, &user_context.user_name)
                .await
                .unwrap();
        }
    }
    HttpResponse::Ok().json(R::<()>::ok_with_msg("退出成功"))
}

/// 获取用户信息
#[get("/getInfo")]
pub async fn get_info(
    req: HttpRequest,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    let claims = req.extensions().get::<Arc<Claims>>().cloned().unwrap();
    let user_id = claims.user_id;

    info!("获取用户信息: user_id={}", user_id);
    match user_service.get_user_by_id(user_id).await {
        Ok(user) => {
            if let Some(user) = user {
                // 获取用户角色信息
                if UserInfo::is_admin(user_id) {
                    let roles = vec![String::from("admin")];
                    let permissions = vec![String::from("*:*:*")];
                    return HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
                            "permissions": permissions,
                            "roles": roles,
                            "user": user
                        }
                    )));
                }
                let roles = user
                    .roles
                    .iter()
                    .filter(|role| !role.role_key.is_empty())
                    .map(|role| role.role_key.clone())
                    .collect::<Vec<String>>();
                // 获取用户权限信息
                let permissions = match user_service.get_user_permissions(user_id).await {
                    Ok(permissions) => permissions,
                    Err(e) => {
                        error!("获取用户权限信息失败: {}", e);
                        vec![]
                    }
                };

                HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
                    "permissions": permissions,
                    "roles": roles,
                    "user": user
                })))
            } else {
                HttpResponse::Ok().json(R::<String>::fail("用户不存在"))
            }
        }
        Err(e) => {
            error!("获取用户信息失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail("获取用户信息失败"))
        }
    }
}

/// 获取路由
#[get("/getRouters")]
pub async fn get_routers(
    req: HttpRequest,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    let claims = req.extensions().get::<Arc<Claims>>().cloned().unwrap();
    let user_id = claims.user_id;

    info!("获取路由: user_id={}", user_id);
    match menu_service.get_user_router_tree(user_id).await {
        Ok(routers) => HttpResponse::Ok().json(RList::ok_with_data(routers)),
        Err(e) => {
            error!("获取路由失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail("获取路由失败"))
        }
    }
}

/// 注册登录路由
pub fn load_login_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(logout)
        .service(get_info)
        .service(get_routers);
}
