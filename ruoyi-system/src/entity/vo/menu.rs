use std::sync::{Arc, RwLock};

use ruoyi_common::utils::tree::TreeNode;
use serde::Serialize;

use crate::entity::prelude::*;

#[derive(Clone, Serialize)]
pub struct MenuSelect {
    pub id: i64,
    #[serde(skip)]
    pub parent_id: i64,
    pub label: String,
    pub disabled: bool,
    pub children: Arc<RwLock<Vec<Arc<MenuSelect>>>>,
}

impl MenuSelect {
    pub fn from_menu_model(menu: &MenuModel) -> Self {
        Self {
            id: menu.menu_id,
            parent_id: menu.parent_id.unwrap_or(0),
            label: menu.menu_name.clone(),
            children: Arc::new(RwLock::new(vec![])),
            disabled: menu.status == Some("1".to_string()),
        }
    }
}

impl TreeNode for MenuSelect {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_parent_id(&self) -> i64 {
        self.parent_id
    }

    fn add_child(&self, child: Arc<Self>) {
        self.children.write().unwrap().push(child);
    }

    fn is_root(&self) -> bool {
        self.parent_id == 0
    }
}
