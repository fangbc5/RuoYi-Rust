use std::sync::{Arc, RwLock};

use ruoyi_common::utils::string::option_is_empty;
use ruoyi_common::utils::tree::{children_is_empty, TreeNode};

use crate::entity::menu::Model as MenuModel;

use super::meta::RouterMeta;

/// 路由前端对象
#[derive(serde::Serialize, Debug, Clone)]
pub struct RouterVo {
    /// 路由id
    #[serde(skip)]
    pub id: i64,
    /// 路由父id
    #[serde(skip)]
    pub parent_id: i64,
    /// 菜单类型
    #[serde(skip)]
    pub menu_type: String,
    /// 路由名字
    pub name: String,
    /// 路由地址
    pub path: String,
    /// 是否隐藏路由，当设置 true 的时候该路由不会在侧边栏出现
    pub hidden: bool,
    /// 重定向地址，当设置 noRedirect 的时候该路由在面包屑导航中不可被点击
    #[serde(skip_serializing_if = "option_is_empty")]
    pub redirect: Option<String>,
    /// 组件地址
    #[serde(skip_serializing_if = "option_is_empty")]
    pub component: Option<String>,
    /// 路由参数：如 {"id": 1, "name": "ry"}
    #[serde(skip_serializing_if = "option_is_empty")]
    pub query: Option<String>,
    /// 当你一个路由下面的 children 声明的路由大于1个时，自动会变成嵌套的模式
    #[serde(rename = "alwaysShow", skip_serializing_if = "always_show_is_none")]
    pub always_show: Arc<RwLock<Option<bool>>>,
    /// 其他元素
    pub meta: RouterMeta,
    /// 子路由
    #[serde(skip_serializing_if = "children_is_empty")]
    pub children: Arc<RwLock<Vec<Arc<RouterVo>>>>,
}
pub fn always_show_is_none(always_show: &Arc<RwLock<Option<bool>>>) -> bool {
    always_show.read().unwrap().is_none()
}

impl RouterVo {
    pub fn from_model(menu: &MenuModel) -> Self {
        //组件处理
        let component = if menu.component.is_none() {
            Some(String::from("Layout"))
        } else if menu.component.as_ref().unwrap().is_empty() {
            Some(String::from("ParentView"))
        } else {
            menu.component.clone()
        };
        let path = if menu.parent_id == Some(0) && menu.is_frame == Some(1) {
            format!("/{}", menu.path.clone().unwrap())
        } else {
            menu.path.clone().unwrap()
        };
        let name = format!(
            "{}{}",
            &menu.path.clone().unwrap()[0..1].to_uppercase(),
            &menu.path.clone().unwrap()[1..]
        );
        let redirect = if menu.menu_type == Some("M".to_string()) && menu.is_frame == Some(1) {
            Some(String::from("noRedirect"))
        } else {
            None
        };
        Self {
            id: menu.menu_id,
            parent_id: menu.parent_id.unwrap_or(0),
            menu_type: menu.menu_type.clone().unwrap_or("".to_string()),
            name,
            path,
            hidden: menu.visible == Some("1".to_string()),
            redirect,
            component,
            query: menu.query.clone(),
            always_show: Arc::new(RwLock::new(None)),
            meta: RouterMeta::from_model(menu),
            children: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl TreeNode for RouterVo {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_parent_id(&self) -> i64 {
        self.parent_id
    }

    fn add_child(&self, child: Arc<Self>) {
        self.children.write().unwrap().push(child);
        if self.menu_type == "M" {
            self.always_show.write().unwrap().replace(true);
        }
    }

    fn is_root(&self) -> bool {
        self.parent_id == 0
    }
}
