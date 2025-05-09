use crate::entity::menu::Model as MenuModel;

/// 路由显示信息
#[derive(serde::Serialize, Debug, Clone)]
pub struct RouterMeta {
    /// 设置该路由在侧边栏和面包屑中展示的名字
    pub title: String,
    /// 设置该路由的图标
    pub icon: String,
    /// 设置为true，则不会被 <keep-alive> 缓存
    #[serde(rename = "noCache")]
    pub no_cache: bool,
    /// 内链地址（http(s)://开头）
    pub link: Option<String>,
}

impl RouterMeta {
    pub fn from_model(menu: &MenuModel) -> Self {
        let link = if menu.is_frame == Some(0) {
            menu.path.clone()
        } else {
            None
        };
        Self {
            title: menu.menu_name.clone(),
            icon: menu.icon.clone().unwrap_or("".to_string()),
            no_cache: menu.is_cache == Some(1),
            link,
        }
    }
}
