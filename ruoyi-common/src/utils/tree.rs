use std::{collections::HashMap, sync::{Arc, RwLock}};

pub trait TreeNode: Send + Sync + Sized + Clone {
    fn get_id(&self) -> i64;
    fn get_parent_id(&self) -> i64;
    fn add_child(&self, child: Arc<Self>);
    fn is_root(&self) -> bool;
}

pub async fn build_tree<T: TreeNode>(nodes: Vec<Arc<T>>) -> Vec<Arc<T>> {
    let mut tree = vec![];
    let mut map = HashMap::new();
    for node in nodes.clone() {
        map.insert(node.get_id(), node);
    }

    for node in nodes {
        if node.is_root() {
            tree.push(node.clone());
            continue;
        }
        if let Some(parent) = map.get_mut(&node.get_parent_id()) {
            parent.add_child(node.clone());
        }
    }

    tree
}

pub fn children_is_empty<T: TreeNode>(children: &Arc<RwLock<Vec<Arc<T>>>>) -> bool {
    children.read().unwrap().is_empty()
}