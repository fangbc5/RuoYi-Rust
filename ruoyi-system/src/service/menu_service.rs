// ruoyi-system/src/service/menu_service.rs
//! 菜单服务实现
use async_trait::async_trait;
use log::error;
use ruoyi_common::error::Error::BusinessError;
use ruoyi_common::utils::tree::build_tree;
use ruoyi_common::vo::PageParam;
use ruoyi_common::Result;
use sea_orm::{ColumnTrait, Condition, IntoActiveModel, Set};
use std::sync::Arc;

use crate::controller::menu_controller::{CreateOrUpdateMenuRequest, MenuQuery};
use crate::entity::vo::menu::MenuSelect;
use crate::entity::vo::user::UserInfo;
use crate::entity::{menu::Column, prelude::*, vo::router::RouterVo};
use crate::repository::menu_repository::MenuRepository;

/// 菜单服务特征
#[async_trait]
pub trait MenuService: Send + Sync {
    /// 获取菜单列表
    async fn get_menu_list(
        &self,
        req: MenuQuery,
        page_param: PageParam,
    ) -> Result<(Vec<MenuModel>, u64)>;

    async fn get_menus_all(&self, req: MenuQuery) -> Result<Vec<MenuModel>>;

    /// 根据菜单ID获取菜单
    async fn get_menu_by_id(&self, menu_id: i64) -> Result<Option<MenuModel>>;

    /// 根据菜单名称获取菜单
    async fn get_menu_by_name(&self, menu_name: &str) -> Result<Option<MenuModel>>;

    /// 创建菜单
    async fn create_menu(&self, menu: CreateOrUpdateMenuRequest) -> Result<MenuModel>;

    /// 更新菜单
    async fn update_menu(&self, menu: CreateOrUpdateMenuRequest) -> Result<MenuModel>;

    /// 删除菜单
    async fn delete_menu(&self, menu_id: i64) -> Result<()>;

    /// 获取角色菜单树
    async fn get_role_menu_tree(&self, role_id: i64) -> Result<Vec<MenuModel>>;

    /// 获取用户菜单树
    async fn get_user_router_tree(&self, user_id: i64) -> Result<Vec<Arc<RouterVo>>>;

    /// 获取菜单选择树
    async fn get_menu_treeselect(&self) -> Vec<Arc<MenuSelect>>;

    /// 根据角色ID获取菜单ID列表
    async fn get_menu_ids_by_role_id(&self, role_id: i64) -> Vec<i64>;
}

/// 菜单服务实现
pub struct MenuServiceImpl {
    menu_repository: Arc<dyn MenuRepository>,
}

impl MenuServiceImpl {
    pub fn new(menu_repository: Arc<dyn MenuRepository>) -> Self {
        Self { menu_repository }
    }

    pub fn build_query_condition(&self, req: &MenuQuery) -> Option<Condition> {
        let mut cond = Condition::all();
        if let Some(name) = &req.menu_name {
            if !name.is_empty() {
                cond = cond.add(MenuColumn::MenuName.contains(name));
            }
        }
        if let Some(status) = &req.status {
            if !status.is_empty() {
                cond = cond.add(Column::Status.eq(status.as_str()));
            }
        }
        Some(cond)
    }
}

#[async_trait]
impl MenuService for MenuServiceImpl {
    async fn get_menu_list(
        &self,
        req: MenuQuery,
        page_param: PageParam,
    ) -> Result<(Vec<MenuModel>, u64)> {
        let condition = self.build_query_condition(&req);
        let (menus, total) = self
            .menu_repository
            .find_list(condition, page_param)
            .await?;
        Ok((menus, total))
    }

    async fn get_menus_all(&self, req: MenuQuery) -> Result<Vec<MenuModel>> {
        let condition = self.build_query_condition(&req);
        Ok(self.menu_repository.find_all(condition).await?)
    }

    async fn get_menu_by_id(&self, menu_id: i64) -> Result<Option<MenuModel>> {
        Ok(self.menu_repository.find_by_id(menu_id).await?)
    }

    async fn get_menu_by_name(&self, menu_name: &str) -> Result<Option<MenuModel>> {
        Ok(self.menu_repository.find_by_name(menu_name).await?)
    }

    async fn create_menu(&self, req: CreateOrUpdateMenuRequest) -> Result<MenuModel> {
        let menu_model = MenuModel {
            menu_id: 0,
            menu_name: req.menu_name.unwrap(),
            parent_id: req.parent_id,
            order_num: req.order_num,
            path: req.path,
            component: req.component,
            query: req.query,
            is_frame: req.is_frame,
            is_cache: req.is_cache,
            menu_type: req.menu_type,
            visible: req.visible,
            status: req.status,
            perms: req.perms,
            icon: req.icon,
            remark: req.remark,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            route_name: None,
        };
        let menu_active_model = menu_model.into_active_model();
        Ok(self.menu_repository.create_menu(menu_active_model).await?)
    }

    async fn update_menu(&self, req: CreateOrUpdateMenuRequest) -> Result<MenuModel> {
        let menu_id = req.menu_id.unwrap();
        let menu_model = self.menu_repository.find_by_id(menu_id).await?;
        if menu_model.is_none() {
            return Err(BusinessError(format!("菜单不存在")));
        }
        let mut menu_active_model = menu_model.unwrap().into_active_model();
        // 更新菜单父级
        if let Some(parent_id) = req.parent_id {
            menu_active_model.parent_id = Set(Some(parent_id));
        }
        // 更新菜单名称
        if let Some(menu_name) = req.menu_name {
            menu_active_model.menu_name = Set(menu_name);
        }
        // 更新菜单路径
        if let Some(path) = req.path {
            menu_active_model.path = Set(Some(path));
        }
        // 更新菜单组件
        if let Some(component) = req.component {
            menu_active_model.component = Set(Some(component));
        }
        // 更新菜单查询条件
        if let Some(query) = req.query {
            menu_active_model.query = Set(Some(query));
        }
        // 更新菜单是否为外链
        if let Some(is_frame) = req.is_frame {
            menu_active_model.is_frame = Set(Some(is_frame));
        }
        // 更新菜单是否缓存
        if let Some(is_cache) = req.is_cache {
            menu_active_model.is_cache = Set(Some(is_cache));
        }
        // 更新菜单类型
        if let Some(menu_type) = req.menu_type {
            menu_active_model.menu_type = Set(Some(menu_type));
        }
        // 更新菜单是否显示
        if let Some(visible) = req.visible {
            menu_active_model.visible = Set(Some(visible));
        }
        // 更新菜单状态
        if let Some(status) = req.status {
            menu_active_model.status = Set(Some(status));
        }
        // 更新菜单权限
        if let Some(perms) = req.perms {
            menu_active_model.perms = Set(Some(perms));
        }
        // 更新菜单图标
        if let Some(icon) = req.icon {
            menu_active_model.icon = Set(Some(icon));
        }
        // 更新菜单备注
        if let Some(remark) = req.remark {
            menu_active_model.remark = Set(Some(remark));
        }

        Ok(self.menu_repository.update_menu(menu_active_model).await?)
    }

    async fn delete_menu(&self, menu_id: i64) -> Result<()> {
        // 检查是否存在子菜单
        let has_child = self
            .menu_repository
            .has_child_by_id(menu_id)
            .await
            .map_err(|e| {
                error!("检查子菜单失败: {}", e);
                BusinessError(format!("检查子菜单失败: {}", e))
            })?;

        if has_child {
            return Err(BusinessError(format!("存在子菜单，不允许删除")));
        }

        // 检查菜单是否已分配
        let is_assigned = self
            .menu_repository
            .check_menu_assigned(menu_id)
            .await
            .map_err(|e| {
                error!("检查菜单分配失败: {}", e);
                BusinessError(format!("检查菜单分配失败: {}", e))
            })?;

        if is_assigned {
            return Err(BusinessError(format!("菜单已分配，不允许删除")));
        }

        Ok(self.menu_repository.delete_by_id(menu_id).await?)
    }

    async fn get_role_menu_tree(&self, _role_id: i64) -> Result<Vec<MenuModel>> {
        // 查询所有菜单
        let menus = self.menu_repository.find_all(None).await.map_err(|e| {
            error!("查询菜单列表失败: {}", e);
            BusinessError(format!("查询菜单列表失败: {}", e))
        })?;

        Ok(menus)
    }

    async fn get_user_router_tree(&self, user_id: i64) -> Result<Vec<Arc<RouterVo>>> {
        // 系统管理员显示所有菜单
        let menus = if UserInfo::is_admin(user_id) {
            self.menu_repository.find_all(None).await?
        } else {
            self.menu_repository
                .select_menus_by_user_id(user_id)
                .await?
        };

        // 将菜单列表转换为路由树
        let router_vos = menus
            .iter()
            .filter(|menu| {
                menu.menu_type == Some("M".to_string()) || menu.menu_type == Some("C".to_string())
            })
            .map(|menu| Arc::new(RouterVo::from_model(menu)))
            .collect();
        let tree = build_tree(router_vos).await;
        Ok(tree)
    }

    async fn get_menu_treeselect(&self) -> Vec<Arc<MenuSelect>> {
        let mut condition = Condition::all();
        condition = condition.add(MenuColumn::Status.eq("0"));
        match self.menu_repository.find_all(Some(condition)).await {
            Ok(menus) => {
                let menu_selects = menus
                    .iter()
                    .map(|menu| Arc::new(MenuSelect::from_menu_model(menu)))
                    .collect();
                let tree = build_tree(menu_selects).await;
                tree
            }
            Err(e) => {
                error!("查询菜单树失败: {}", e);
                vec![]
            }
        }
    }

    async fn get_menu_ids_by_role_id(&self, role_id: i64) -> Vec<i64> {
        match self
            .menu_repository
            .select_menu_ids_by_role_id(role_id)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                error!("查询菜单ID列表失败: {}", e);
                vec![]
            }
        }
    }
}
