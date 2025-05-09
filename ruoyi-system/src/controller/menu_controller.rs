// ruoyi-system/src/controller/menu_controller.rs
//! 菜单管理控制器

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;

use crate::service::menu_service::{MenuService, MenuServiceImpl};
use ruoyi_common::utils::string::{deserialize_str_to_i32, option_is_empty};
use ruoyi_common::vo::{RData, RList, R};

/// 菜单查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuQuery {
    /// 菜单名称
    pub menu_name: Option<String>,
    /// 状态
    pub status: Option<String>,
}

/// 创建菜单请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateMenuRequest {
    /// 菜单ID
    pub menu_id: Option<i64>,
    /// 菜单名称
    pub menu_name: Option<String>,
    /// 父菜单ID
    pub parent_id: Option<i64>,
    /// 显示顺序
    pub order_num: Option<i32>,
    /// 路由地址
    pub path: Option<String>,
    /// 组件路径
    pub component: Option<String>,
    /// 查询条件
    pub query: Option<String>,
    /// 是否为外链（0否 1是）
    #[serde(deserialize_with = "deserialize_str_to_i32")]
    pub is_frame: Option<i32>,
    /// 是否缓存（0否 1是）
    #[serde(deserialize_with = "deserialize_str_to_i32")]
    pub is_cache: Option<i32>,
    /// 菜单类型（M目录 C菜单 F按钮）
    pub menu_type: Option<String>,
    /// 是否显示（0不显示 1显示）
    pub visible: Option<String>,
    /// 状态（0停用 1正常）
    pub status: Option<String>,
    /// 权限标识
    pub perms: Option<String>,
    /// 菜单图标
    pub icon: Option<String>,
    /// 备注
    pub remark: Option<String>,
}

/// 获取菜单列表
#[get("/list")]
pub async fn list_menus(
    req: web::Query<MenuQuery>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    info!("查询菜单列表: {:?}", req.0);

    match menu_service.get_menus_all(req.0).await {
        Ok(menus) => HttpResponse::Ok().json(RList::ok_with_data(menus)),
        Err(e) => {
            error!("查询菜单列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询菜单列表失败: {}", e)))
        }
    }
}

/// 获取菜单详情
#[get("/{id}")]
pub async fn get_menu(
    menu_id: web::Path<i64>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    info!("获取菜单详情, id: {}", menu_id);

    match menu_service.get_menu_by_id(*menu_id).await {
        Ok(menu) => HttpResponse::Ok().json(RData::ok(menu)),
        Err(e) => {
            error!("获取菜单详情失败: {}", e);
            HttpResponse::InternalServerError().json(R::<String>::fail(&e.to_string()))
        }
    }
}

/// 创建菜单
#[post("")]
pub async fn create_menu(
    req: web::Json<CreateOrUpdateMenuRequest>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    let (valid, msg) = check_menu_valid("create", &req.0, &menu_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&msg.unwrap()));
    }
    info!("创建菜单: {:?}", req.0);
    match menu_service.create_menu(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建成功")),
        Err(e) => {
            error!("创建菜单失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建菜单失败: {}", e)))
        }
    }
}

/// 更新菜单
#[put("")]
pub async fn update_menu(
    req: web::Json<CreateOrUpdateMenuRequest>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    let (valid, msg) = check_menu_valid("update", &req.0, &menu_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&msg.unwrap()));
    }
    info!("更新菜单: {:?}", req.0);
    match menu_service.update_menu(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("更新成功")),
        Err(e) => {
            error!("更新菜单失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新菜单失败: {}", e)))
        }
    }
}

/// 删除菜单
#[delete("/{id}")]
pub async fn delete_menu(
    menu_id: web::Path<i64>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    let menu_id = menu_id.into_inner();
    info!("删除菜单, id: {}", menu_id);
    match menu_service.delete_menu(menu_id).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除成功")),
        Err(e) => {
            error!("删除菜单失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除菜单失败: {}", e)))
        }
    }
}

/// 获取菜单树
#[get("/treeselect")]
pub async fn get_menu_treeselect(menu_service: web::Data<MenuServiceImpl>) -> impl Responder {
    info!("获取菜单树");
    HttpResponse::Ok().json(RList::ok_with_data(
        menu_service.get_menu_treeselect().await,
    ))
}

/// 获取菜单树
#[get("/roleMenuTreeselect/{roleId}")]
pub async fn get_role_menu_treeselect(
    role_id: web::Path<i64>,
    menu_service: web::Data<MenuServiceImpl>,
) -> impl Responder {
    let role_id = role_id.into_inner();
    info!("获取菜单树");
    let menu_ids = menu_service.get_menu_ids_by_role_id(role_id).await;
    let menu_tree = menu_service.get_menu_treeselect().await;
    HttpResponse::Ok().json(R::ok_with_data(serde_json::json!({
        "checkedKeys": menu_ids,
        "menus": menu_tree,
    })))
}

pub async fn check_menu_valid(
    action: &str,
    req: &CreateOrUpdateMenuRequest,
    menu_service: &MenuServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" {
        if req.menu_id.is_none() {
            return (false, Some("菜单ID不能为空".to_string()));
        }
    }
    let menu_type = req.menu_type.as_ref().unwrap();
    if menu_type == "M" || menu_type == "C" {
        // 路由地址
        if option_is_empty(&req.path) {
            return (false, Some("路由地址不能为空".to_string()));
        }
        // 是否外链
        if req.is_frame.is_none() {
            return (false, Some("是否外链不能为空".to_string()));
        }
        // 显示状态
        if option_is_empty(&req.visible) {
            return (false, Some("显示状态不能为空".to_string()));
        }
    }

    if req.parent_id.is_none() {
        return (false, Some("父菜单不能为空".to_string()));
    }
    if option_is_empty(&req.menu_name) {
        return (false, Some("菜单名称不能为空".to_string()));
    }
    if option_is_empty(&req.menu_type) {
        return (false, Some("菜单类型不能为空".to_string()));
    }
    // 排序
    if req.order_num.is_none() {
        return (false, Some("排序不能为空".to_string()));
    }
    // 菜单状态
    if option_is_empty(&req.status) {
        return (false, Some("菜单状态不能为空".to_string()));
    }
    if action == "create" {
        // 菜单名称不能重复
        match menu_service
            .get_menu_by_name(req.menu_name.as_ref().unwrap())
            .await
        {
            Ok(Some(_)) => return (false, Some("菜单名称不能重复".to_string())),
            Ok(None) => (),
            Err(e) => {
                error!("查询菜单名称失败: {}", e);
                return (false, Some("查询菜单名称失败".to_string()));
            }
        }
    }
    (true, None)
}

/// 注册菜单控制器路由
pub fn load_menu_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/menu")
            .service(list_menus)
            .service(create_menu)
            .service(update_menu)
            .service(delete_menu)
            .service(get_menu_treeselect)
            .service(get_role_menu_treeselect)
            .service(get_menu),
    );
}
