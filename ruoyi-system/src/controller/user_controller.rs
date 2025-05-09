// ruoyi-system/src/controller/user_controller.rs
//! 用户管理控制器

use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use ruoyi_common::utils::{string::option_is_empty, time::deserialize_optional_datetime};
use serde::{Deserialize, Serialize};

use ruoyi_common::{
    utils::jwt::Claims,
    vo::{PageParam, RList, R},
};

use crate::service::{
    dept_service::{DeptService, DeptServiceImpl},
    post_service::{PostService, PostServiceImpl},
    role_service::{RoleService, RoleServiceImpl},
    user_service::{UserService, UserServiceImpl},
};

/// 用户查询参数
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuery {
    /// 用户ID
    pub user_id: Option<i64>,

    /// 部门ID
    pub dept_id: Option<i64>,

    /// 用户账号
    pub user_name: Option<String>,

    /// 用户昵称
    pub nick_name: Option<String>,

    /// 手机号码
    pub phonenumber: Option<String>,

    /// 帐号状态（0正常 1停用）
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

/// 创建用户请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateUserRequest {
    /// 用户ID
    pub user_id: Option<i64>,
    /// 用户名
    pub user_name: Option<String>,
    /// 昵称
    pub nick_name: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 手机号
    pub phonenumber: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 性别（0-未知, 1-男, 2-女）
    pub sex: Option<String>,
    /// 状态（0-禁用, 1-正常）
    pub status: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 部门ID
    pub dept_id: Option<i64>,
    /// 角色ID列表
    pub role_ids: Option<Vec<i64>>,
    /// 岗位ID列表
    pub post_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    /// 用户id
    pub user_id: i64,
    /// 新密码
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeStatusRequest {
    /// 用户id
    pub user_id: i64,
    /// 新密码
    pub status: String,
}

/// 获取用户列表
#[get("/list")]
pub async fn list_users(
    query: web::Query<UserQuery>,
    page_param: web::Query<PageParam>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    info!("查询用户列表: {:?}", query);

    // 这里简化调用方式，与service接口匹配
    match user_service.get_user_list(query.0, page_param.0).await {
        Ok((users, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "total": total,
            "rows": users,
        }))),
        Err(e) => {
            error!("查询用户列表失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail("查询用户列表失败"))
        }
    }
}

/// 获取用户详情
#[get("/{id}")]
pub async fn get_user(
    path: web::Path<i64>,
    user_service: web::Data<UserServiceImpl>,
    post_service: web::Data<PostServiceImpl>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let user_id = path.into_inner();
    info!("获取用户详情: {}", user_id);

    match user_service.get_user_by_id(user_id).await {
        Ok(user) => {
            if let Some(user) = user {
                // 获取岗位信息
                let post_ids = post_service.get_post_ids_by_user_id(user.user_id).await;
                let posts = post_service.get_posts_all().await;
                let roles = role_service.get_roles_all().await;
                let role_ids: Vec<i64> = user.roles.iter().map(|role| role.role_id).collect();
                HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
                    "data": user,
                    "posts": posts,
                    "roles": roles,
                    "roleIds": role_ids,
                    "postIds": post_ids,
                })))
            } else {
                HttpResponse::Ok().json(R::<String>::fail("用户不存在"))
            }
        }
        Err(e) => {
            error!("获取用户详情失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail("获取用户详情失败"))
        }
    }
}

/// 创建用户
#[post("")]
pub async fn create_user(
    req: web::Json<CreateOrUpdateUserRequest>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_user_valid("create", &req.0, &user_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("创建用户: {:?}", req);
    // 由于我们无法完全确定service层的接口，这里采用最简单的转发形式
    match user_service.create_user(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建用户成功")),
        Err(e) => {
            error!("创建用户失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("创建用户失败: {}", e)))
        }
    }
}

/// 更新用户
#[put("")]
pub async fn update_user(
    req: web::Json<CreateOrUpdateUserRequest>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    let user_id = req.user_id;
    // 检查用户名id是否为空
    if user_id.is_none() {
        return HttpResponse::Ok().json(R::<String>::fail("用户名不能为空"));
    }
    let (is_valid, error_msg) = check_user_valid("update", &req.0, &user_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("更新用户: req={:?}", req);
    match user_service.update_user(req.0).await {
        Ok(user) => {
            info!("更新用户成功: user={:?}", user);
            HttpResponse::Ok().json(R::<String>::ok_with_msg("更新用户成功"))
        }
        Err(e) => {
            error!("更新用户失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新用户失败: {}", e)))
        }
    }
}

/// 删除用户
#[delete("/{ids}")]
pub async fn delete_user(
    ids: web::Path<String>,
    req: HttpRequest,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    let claims = req.extensions().get::<Arc<Claims>>().cloned().unwrap();
    let current_user_id = claims.user_id;

    // 解析逗号分隔的ID字符串为Vec<i64>
    let ids_str = ids.into_inner();
    let parsed_ids: Result<Vec<i64>, _> = ids_str
        .split(',')
        .map(|id| id.trim().parse::<i64>())
        .collect();

    match parsed_ids {
        Ok(ids) => {
            if ids.contains(&current_user_id) {
                return HttpResponse::BadRequest().json(R::<String>::fail("不能删除当前用户"));
            }
            info!("删除用户: {:?}", ids);

            match user_service.delete_user_by_ids(ids).await {
                Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除用户成功")),
                Err(e) => {
                    error!("删除用户失败: {}", e);
                    HttpResponse::InternalServerError()
                        .json(R::<String>::fail(&format!("删除用户失败: {}", e)))
                }
            }
        }
        Err(e) => {
            error!("解析用户ID失败: {}", e);
            HttpResponse::BadRequest()
                .json(R::<String>::fail(&format!("无效的用户ID: {}", ids_str)))
        }
    }
}

/// 重置密码
#[put("/resetPwd")]
pub async fn reset_password(
    req: web::Json<ResetPasswordRequest>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    info!("重置用户密码: id={}", req.user_id);

    match user_service
        .reset_password(req.user_id, &req.password)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("重置密码成功")),
        Err(e) => {
            error!("重置密码失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("重置密码失败: {}", e)))
        }
    }
}

/// 修改状态
#[put("/changeStatus")]
pub async fn change_status(
    req: web::Json<ChangeStatusRequest>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    info!("修改用户状态: id={}", req.user_id);

    match user_service
        .update_user_status(req.user_id, &req.status)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("修改状态成功")),
        Err(e) => {
            error!("修改状态失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("修改状态失败: {}", e)))
        }
    }
}

/// 部门树
#[get("/deptTree")]
pub async fn get_dept_tree(dept_service: web::Data<DeptServiceImpl>) -> impl Responder {
    match dept_service.get_dept_tree().await {
        Ok(dept_tree) => HttpResponse::Ok().json(RList::ok_with_data(dept_tree)),
        Err(e) => {
            error!("获取部门树失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail("获取部门树失败"))
        }
    }
}

/// 新增用户时获取岗位和角色
#[get("/")]
pub async fn get_post_and_role(
    post_service: web::Data<PostServiceImpl>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let posts = post_service.get_posts_all().await;
    let roles = role_service.get_roles_all().await;
    HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
        "posts": posts,
        "roles": roles,
    })))
}

#[get("/authRole/{userId}")]
pub async fn auth_role(
    user_id: web::Path<i64>,
    user_service: web::Data<UserServiceImpl>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    match user_service.get_user_by_id(user_id).await {
        Ok(user) => {
            if let Some(user) = user {
                HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
                    "roles": user.roles,
                    "user": user,
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

async fn check_user_valid(
    action: &str,
    req: &CreateOrUpdateUserRequest,
    user_service: &UserServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" && req.user_id.is_none() {
        return (false, Some("用户ID不能为空".to_string()));
    }
    // 检查用户名是否为空，且不存在
    if option_is_empty(&req.user_name) {
        return (false, Some("用户名不能为空".to_string()));
    }
    // 检查昵称
    if option_is_empty(&req.nick_name) {
        return (false, Some("昵称不能为空".to_string()));
    }
    // 检查密码是否为空
    if option_is_empty(&req.password) {
        return (false, Some("密码不能为空".to_string()));
    }

    // 用户名重复性校验
    match user_service
        .check_user_name_unique(&req.user_name.as_ref().unwrap(), req.user_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("用户名已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查用户名是否重复失败: {}", e);
            return (false, Some("检查用户名是否重复失败".to_string()));
        }
    }
    (true, None)
}

/// 注册用户控制器路由
pub fn load_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(list_users)
            .service(reset_password)
            .service(change_status)
            .service(create_user)
            .service(update_user)
            .service(delete_user)
            .service(get_dept_tree)
            .service(get_user)
            .service(get_post_and_role)
            .service(auth_role),
    );
}
