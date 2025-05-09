use std::sync::{Arc, RwLock};

use ruoyi_common::utils::tree::TreeNode;
use serde::Serialize;

use crate::entity::prelude::*;

#[derive(Clone, Serialize)]
pub struct DeptSelect {
    pub id: i64,
    #[serde(skip)]
    pub parent_id: i64,
    pub label: String,
    pub disabled: bool,
    pub children: Arc<RwLock<Vec<Arc<DeptSelect>>>>,
}

impl DeptSelect {
    pub fn from_dept_model(dept: &DeptModel) -> Self {
        Self {
            id: dept.dept_id,
            parent_id: dept.parent_id.unwrap_or(0),
            label: dept.dept_name.clone().unwrap_or_default(),
            children: Arc::new(RwLock::new(vec![])),
            disabled: dept.status.clone().unwrap_or_default() == "1".to_string(),
        }
    }
}

impl TreeNode for DeptSelect {
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
