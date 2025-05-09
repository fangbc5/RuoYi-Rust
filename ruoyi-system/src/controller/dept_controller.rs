// ruoyi-system/src/controller/dept_controller.rs
//! 部门管理控制器

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use log::{error, info};
use serde::{Deserialize, Serialize};

use ruoyi_common::{utils::string::option_is_empty, vo::{RData, RList, R}};

use crate::service::dept_service::{DeptService, DeptServiceImpl};

/// 部门查询参数
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptQuery {
    /// 部门名称
    pub dept_name: Option<String>,

    /// 部门状态（0正常 1停用）
    pub status: Option<String>,
}

/// 创建部门请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateDeptRequest {
    /// 部门ID
    pub dept_id: Option<i64>,

    /// 父部门ID
    pub parent_id: Option<i64>,

    /// 部门名称
    pub dept_name: Option<String>,

    /// 显示顺序
    pub order_num: Option<i32>,

    /// 负责人
    pub leader: Option<String>,

    /// 联系电话
    pub phone: Option<String>,

    /// 邮箱
    pub email: Option<String>,

    /// 部门状态（0正常 1停用）
    pub status: Option<String>,
}

/// 获取部门列表
#[get("/list")]
pub async fn list_depts(
    req: web::Query<DeptQuery>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    info!("查询部门列表: {:?}", req);

    match dept_service.get_dept_list(req.0, None).await {
        Ok(depts) => HttpResponse::Ok().json(RList::ok_with_data(depts)),
        Err(e) => {
            error!("查询部门列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询部门列表失败: {}", e)))
        }
    }
}

#[get("/list/exclude/{id}")]
pub async fn list_depts_exclude_self(
    path: web::Path<i64>,
    req: web::Query<DeptQuery>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let dept_id = path.into_inner();
    info!("查询部门列表: {}", dept_id);

    match dept_service.get_dept_list(req.0, Some(dept_id)).await {
        Ok(depts) => HttpResponse::Ok().json(RList::ok_with_data(depts)),
        Err(e) => {
            error!("查询部门列表失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("查询部门列表失败: {}", e)))
        }
    }
}
/// 获取部门详情
#[get("/{id}")]
pub async fn get_dept(
    path: web::Path<i64>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let dept_id = path.into_inner();
    info!("获取部门详情: {}", dept_id);

    match dept_service.get_dept_by_id(dept_id).await {
        Ok(dept) => HttpResponse::Ok().json(RData::ok(dept)),
        Err(e) => {
            error!("获取部门详情失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("获取部门详情失败: {}", e)))
        }
    }
}

/// 创建部门
#[post("")]
pub async fn create_dept(
    req: web::Json<CreateOrUpdateDeptRequest>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let (valid, msg) = check_dept_valid("create", &req.0, &dept_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&msg.unwrap()));
    }
    info!("创建部门: {:?}", req);
    match dept_service.create_dept(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("创建部门成功")),
        Err(e) => {
            error!("创建部门失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("创建部门失败: {}", e)))
        }
    }
}

/// 更新部门
#[put("")]
pub async fn update_dept(
    req: web::Json<CreateOrUpdateDeptRequest>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let (valid, msg) = check_dept_valid("update", &req.0, &dept_service).await;
    if !valid {
        return HttpResponse::Ok().json(R::<String>::fail(&msg.unwrap()));
    }
    info!("更新部门: req={:?}", req);

    match dept_service.update_dept(req.0).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("更新部门成功")),
        Err(e) => {
            error!("更新部门失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("更新部门失败: {}", e)))
        }
    }
}

/// 删除部门    
#[delete("/{id}")]
pub async fn delete_dept(
    path: web::Path<i64>,
    dept_service: web::Data<DeptServiceImpl>,
) -> impl Responder {
    let dept_id = path.into_inner();
    info!("删除部门: {}", dept_id);

    match dept_service.delete_dept(dept_id).await {
        Ok(_) => HttpResponse::Ok().json(R::<String>::ok_with_msg("删除部门成功")),
        Err(e) => {
            error!("删除部门失败: {}", e);
            HttpResponse::Ok().json(R::<String>::fail(&format!("删除部门失败: {}", e)))
        }
    }
}

async fn check_dept_valid(
    action: &str,
    req: &CreateOrUpdateDeptRequest,
    dept_service: &DeptServiceImpl,
) -> (bool, Option<String>) {
    if action == "update" && req.dept_id.is_none() {
        return (false, Some("部门ID不能为空".to_string()));
    }

    if option_is_empty(&req.dept_name) {
        return (false, Some("部门名称不能为空".to_string()));
    }

    if req.parent_id.is_none() {
        return (false, Some("父部门不能为空".to_string()));
    }
    if req.order_num.is_none() {
        return (false, Some("显示顺序不能为空".to_string()));
    }
    if option_is_empty(&req.status) {
        return (false, Some("状态不能为空".to_string()));
    }

    match dept_service
        .check_dept_name_unique(
            req.dept_name.as_ref().unwrap(),
            req.parent_id.unwrap(),
            req.dept_id,
        )
        .await
    {
        Ok(vaild) => {
            if !vaild {
                return (false, Some("部门名称已存在".to_string()));
            }
        }
        Err(e) => {
            return (false, Some(format!("部门名称重复校验失败: {}", e)));
        }
    }

    (true, None)
}

/// 注册部门控制器路由
pub fn load_dept_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dept")
            .service(list_depts)
            .service(list_depts_exclude_self)
            .service(create_dept)
            .service(update_dept)
            .service(delete_dept)
            .service(get_dept),
    );
}
