// ruoyi-system/src/controller/role_controller.rs
//! 角色管理控制器

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use ruoyi_common::utils::string::option_is_empty;
use ruoyi_common::utils::time::deserialize_optional_datetime;
use ruoyi_common::vo::{PageParam, RData, R};
use serde::Deserialize;

use crate::service::dept_service::{DeptService, DeptServiceImpl};
use crate::service::role_service::{RoleService, RoleServiceImpl};

/// 角色查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQuery {
    /// 角色名称
    pub role_name: Option<String>,
    /// 权限字符
    pub role_key: Option<String>,
    /// 状态
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
pub struct RoleAuthUserQuery {
    /// 角色ID
    pub role_id: i64,
    /// 用户名
    pub user_name: Option<String>,
    /// 手机号码
    pub phonenumber: Option<String>,
    /// 分页
    pub page_num: u64,
    /// 分页大小
    pub page_size: u64,
}

/// 创建角色请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateRoleRequest {
    /// 角色ID
    pub role_id: Option<i64>,
    /// 角色名称
    pub role_name: Option<String>,
    /// 权限字符
    pub role_key: Option<String>,
    /// 显示顺序
    pub role_sort: Option<i32>,
    /// 数据范围（1：全部数据权限；2：自定义数据权限；3：本部门数据权限；4：本部门及以下数据权限；5：仅本人数据权限）
    pub data_scope: Option<String>,
    /// 菜单树选择项是否关联显示
    pub menu_check_strictly: Option<bool>,
    /// 部门树选择项是否关联显示
    pub dept_check_strictly: Option<bool>,
    /// 状态（0-停用, 1-正常）
    pub status: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 菜单ID列表
    pub menu_ids: Option<Vec<i64>>,
    /// 部门ID列表
    pub dept_ids: Option<Vec<i64>>,
}

/// 角色状态更新请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRoleStatusRequest {
    /// 角色ID
    pub role_id: i64,
    /// 状态（0-停用, 1-正常）
    pub status: String,
}

/// 角色授权用户请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserRoleRequest {
    /// 角色ID
    pub role_id: i64,
    /// 用户ID列表
    pub user_ids: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelSingleUserRoleRequest {
    /// 角色ID
    pub role_id: String,
    /// 用户ID列表
    pub user_id: i64,
}

/// 获取角色列表
#[get("/list")]
pub async fn list_roles(
    query: web::Query<RoleQuery>,
    page_param: web::Query<PageParam>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    info!("查询角色列表: {:?}", query);

    match role_service.get_role_list(query.0, page_param.0).await {
        Ok((roles, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "total": total,
            "rows": roles,
        }))),
        Err(e) => {
            error!("查询角色列表失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail("查询角色列表失败"))
        }
    }
}

/// 获取角色详情
#[get("/{id}")]
pub async fn get_role(
    path: web::Path<i64>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let role_id = path.into_inner();
    info!("获取角色详情: {}", role_id);

    match role_service.get_role_by_id(role_id).await {
        Ok(role) => HttpResponse::Ok().json(RData::ok(role)),
        Err(e) => {
            error!("获取角色详情失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail("获取角色详情失败"))
        }
    }
}

/// 创建角色
#[post("")]
pub async fn create_role(
    req: web::Json<CreateOrUpdateRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_role_valid("create", &req.0, &role_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("创建角色: {:?}", req);

    match role_service.create_role(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建角色成功")),
        Err(e) => {
            error!("创建角色失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("创建角色失败: {}", e)))
        }
    }
}

/// 更新角色
#[put("")]
pub async fn update_role(
    req: web::Json<CreateOrUpdateRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let (is_valid, error_msg) = check_role_valid("update", &req.0, &role_service).await;
    if !is_valid {
        return HttpResponse::Ok().json(R::<String>::fail(&error_msg.unwrap()));
    }
    info!("更新角色: req={:?}", req);
    match role_service.update_role(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("更新角色成功")),
        Err(e) => {
            error!("更新角色失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新角色失败: {}", e)))
        }
    }
}

/// 删除角色
#[delete("/{ids}")]
pub async fn delete_roles(
    ids: web::Path<String>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    let role_ids = ids.into_inner();
    if role_ids.is_empty() {
        return HttpResponse::Ok().json(R::<String>::fail("角色ID不能为空"));
    }
    let role_ids: Vec<i64> = role_ids
        .split(',')
        .filter_map(|id| id.parse::<i64>().ok())
        .collect();
    info!("删除角色: ids={:?}", role_ids);

    match role_service.delete_roles(role_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除角色成功")),
        Err(e) => {
            error!("删除角色失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("删除角色失败: {}", e)))
        }
    }
}

/// 修改角色状态
#[put("/changeStatus")]
pub async fn change_role_status(
    req: web::Json<ChangeRoleStatusRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    info!("修改角色状态: id={}, status={}", req.role_id, req.status);

    match role_service
        .change_role_status(req.role_id, &req.status)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("修改角色状态成功")),
        Err(e) => {
            error!("修改角色状态失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("修改角色状态失败: {}", e)))
        }
    }
}
/// 获取角色分配的用户列表
#[get("/allocatedList")]
pub async fn get_role_auth_users(
    req: web::Query<RoleAuthUserQuery>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    info!("获取角色分配的用户列表: req={:?}", req);

    match role_service.get_role_allocated_users(req.0).await {
        Ok((users, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "total": total,
            "rows": users,
        }))),
        Err(e) => {
            error!("获取角色分配的用户列表失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail("获取角色分配的用户列表失败"))
        }
    }
}

/// 获取角色未分配的用户列表
#[get("/unallocatedList")]
pub async fn get_role_unallocated_users(
    req: web::Query<RoleAuthUserQuery>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    info!("获取角色未分配的用户列表: req={:?}", req);

    match role_service.get_role_unallocated_users(req.0).await {
        Ok((users, total)) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
            "total": total,
            "rows": users,
        }))),
        Err(e) => {
            error!("获取角色未分配的用户列表失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail("获取角色未分配的用户列表失败"))
        }
    }
}

/// 给角色分配用户
#[put("/selectAll")]
pub async fn auth_role_users(
    req: web::Query<AuthUserRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    if req.role_id == 0 {
        return HttpResponse::BadRequest().json(R::<String>::fail("角色ID不能为空"));
    }
    // 校验
    if req.user_ids.is_empty() {
        return HttpResponse::BadRequest().json(R::<String>::fail("用户ID列表不能为空"));
    }
    let user_ids: Vec<i64> = req
        .user_ids
        .split(',')
        .filter_map(|id| id.parse::<i64>().ok())
        .collect();
    info!(
        "给角色分配用户: role_id={}, user_ids={}",
        req.role_id, req.user_ids
    );
    match role_service.auth_role_users(req.role_id, user_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("分配用户成功")),
        Err(e) => {
            error!("分配用户失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("分配用户失败: {}", e)))
        }
    }
}

/// 取消角色用户授权
#[put("/cancel")]
pub async fn cancel_role_user(
    req: web::Json<CancelSingleUserRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    info!("取消角色用户授权: req={:?}", req);
    if req.role_id.is_empty() {
        return HttpResponse::BadRequest().json(R::<String>::fail("角色ID不能为空"));
    }
    if req.user_id == 0 {
        return HttpResponse::BadRequest().json(R::<String>::fail("用户ID不能为空"));
    }
    let role_id: i64 = match req.role_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::Ok().json(R::<String>::fail("角色ID格式错误")),
    };
    match role_service.cancel_auth_user(role_id, req.user_id).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("取消授权成功")),
        Err(e) => {
            error!("取消授权失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("取消授权失败: {}", e)))
        }
    }
}

/// 批量取消角色用户授权
#[put("/cancelAll")]
pub async fn cancel_role_users(
    req: web::Query<AuthUserRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    if req.role_id == 0 {
        return HttpResponse::BadRequest().json(R::<String>::fail("角色ID不能为空"));
    }
    if req.user_ids.is_empty() {
        return HttpResponse::BadRequest().json(R::<String>::fail("用户ID列表不能为空"));
    }
    let user_ids: Vec<i64> = req
        .user_ids
        .split(',')
        .filter_map(|id| id.parse::<i64>().ok())
        .collect();
    info!(
        "批量取消角色用户授权: role_id={}, user_ids={}",
        req.role_id, req.user_ids
    );

    match role_service.cancel_auth_users(req.role_id, user_ids).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("批量取消授权成功")),
        Err(e) => {
            error!("批量取消授权失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("批量取消授权失败: {}", e)))
        }
    }
}

#[put("/dataScope")]
pub async fn data_scope(
    req: web::Json<CreateOrUpdateRoleRequest>,
    role_service: web::Data<RoleServiceImpl>,
) -> impl Responder {
    if req.role_id.is_none() {
        return HttpResponse::BadRequest().json(R::<String>::fail("角色ID不能为空"));
    }
    if req.data_scope.is_none() {
        return HttpResponse::BadRequest().json(R::<String>::fail("数据范围不能为空"));
    }

    info!("修改角色数据范围: req={:?}", req);

    match role_service.update_role(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("修改角色数据范围成功")),
        Err(e) => {
            error!("修改角色数据范围失败: {}", e);
            HttpResponse::InternalServerError()
                .json(R::<String>::fail(&format!("修改角色数据范围失败: {}", e)))
        }
    }
}

/// 获取部门树
#[get("/deptTree/{role_id}")]
pub async fn get_dept_tree(
    path: web::Path<i64>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let role_id = path.into_inner();
    match dept_service.get_dept_tree().await {
        Ok(dept_tree) => match dept_service.get_dept_ids_by_role_id(role_id).await {
            Ok(dept_ids) => HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
                "depts": dept_tree,
                "checkedKeys": dept_ids,
            }))),
            Err(e) => {
                error!("获取角色部门失败: {}", e);
                HttpResponse::Ok().json(R::<String>::fail(&format!("获取角色部门失败: {}", e)))
            }
        },
        Err(e) => {
            error!("获取部门树失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取部门树失败: {}", e)))
        }
    }
}

/// 检查角色是否有效
async fn check_role_valid(
    action: &str,
    req: &CreateOrUpdateRoleRequest,
    role_service: &RoleServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" && req.role_id.is_none() {
        return (false, Some("角色ID不能为空".to_string()));
    }
    // 检查角色名称是否为空，且不存在
    if option_is_empty(&req.role_name) {
        return (false, Some("角色名称不能为空".to_string()));
    }

    if option_is_empty(&req.role_key) {
        return (false, Some("角色权限字符不能为空".to_string()));
    }

    if req.role_sort.is_none() {
        return (false, Some("角色排序不能为空".to_string()));
    }

    match role_service
        .check_role_name_unique(req.role_name.as_ref().unwrap(), req.role_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("角色名称已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查角色名称是否重复失败: {}", e);
            return (false, Some("检查角色名称是否重复失败".to_string()));
        }
    }

    match role_service
        .check_role_key_unique(req.role_key.as_ref().unwrap(), req.role_id)
        .await
    {
        Ok(is_unique) => {
            if !is_unique {
                return (false, Some("角色权限字符已存在".to_string()));
            }
        }
        Err(e) => {
            error!("检查角色权限字符是否重复失败: {}", e);
            return (false, Some("检查角色权限字符是否重复失败".to_string()));
        }
    }

    return (true, None);
}
/// 注册角色控制器路由
pub fn load_role_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/role")
            .service(list_roles)
            .service(create_role)
            .service(update_role)
            .service(delete_roles)
            .service(change_role_status)
            .service(
                web::scope("/authUser")
                    .service(get_role_auth_users)
                    .service(get_role_unallocated_users)
                    .service(auth_role_users)
                    .service(cancel_role_user)
                    .service(cancel_role_users),
            )
            .service(data_scope)
            .service(get_dept_tree)
            .service(get_role),
    );
}
