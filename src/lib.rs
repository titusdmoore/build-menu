mod utils;

use serde::{Deserialize, Serialize};
use serde_json;
use serde_wasm_bindgen;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Deserialize)]
struct RawMenuItem {
    id: usize,
    name: String,
    url: String,
    parentId: Option<usize>,
}

#[wasm_bindgen]
#[derive(Clone, Serialize)]
struct MenuItem {
    id: usize,
    name: String,
    url: String,
    children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn new(id: usize, name: String, url: String, children: Option<&Vec<MenuItem>>) -> Self {
        if let Some(children) = children {
            return Self {
                id,
                name,
                url,
                children: children.to_vec(),
            };
        }

        Self {
            id,
            name,
            url,
            children: Vec::new(),
        }
    }
}

struct Result {
    nodes_map: HashMap<usize, Vec<usize>>,
    nodes: Vec<MenuItem>,
    holding_nodes: HashMap<usize, Vec<MenuItem>>,
}

impl Result {
    pub fn new() -> Self {
        Self {
            nodes_map: HashMap::new(),
            nodes: Vec::new(),
            holding_nodes: HashMap::new(),
        }
    }
}

#[wasm_bindgen]
pub extern "C" fn build_menu(input: String) -> JsValue {
    if let Ok(raw_menu_items) = serde_json::from_str::<Vec<RawMenuItem>>(&input) {
        let mut result: Result = Result::new();

        let mut position: usize = 0;
        while position == raw_menu_items.len() - 1 {
            position = walk(&mut result, position, &raw_menu_items);
        }

        return serde_wasm_bindgen::to_value(&result.nodes).unwrap();
    }

    serde_wasm_bindgen::to_value(&Vec::<MenuItem>::new()).unwrap()
}

fn walk(result: &mut Result, position: usize, items: &Vec<RawMenuItem>) -> usize {
    if position >= items.len() || position >= 1000 {
        return position;
    }

    let working_item: &RawMenuItem = &items[position];

    // Node doesn't have parent
    if !working_item.parentId.is_some() {
        result.nodes.push(MenuItem::new(
            working_item.id,
            working_item.name.clone(),
            working_item.url.clone(),
            result.holding_nodes.get(&working_item.id),
        ));
        result
            .nodes_map
            .insert(working_item.id, vec![result.nodes.len() as usize - 1]);
    // Node has parent
    } else {
        if let Some(parent_id) = working_item.parentId {
            // Node's parent already is in node arr
            if result.nodes_map.contains_key(&parent_id) {
                let new_node = MenuItem::new(
                    working_item.id,
                    working_item.name.clone(),
                    working_item.url.clone(),
                    result.holding_nodes.get(&working_item.id),
                );
                add_to_parent(result, new_node, parent_id);
            } else {
                let new_node = MenuItem::new(
                    working_item.id,
                    working_item.name.clone(),
                    working_item.url.clone(),
                    result.holding_nodes.get(&working_item.id),
                );
                if let Some(awaiting_node_vec) = result.holding_nodes.get_mut(&parent_id) {
                    awaiting_node_vec.push(new_node);
                } else {
                    result.holding_nodes.insert(parent_id, vec![new_node]);
                }
            }
        }
    }

    return walk(result, position + 1, items);
}

fn add_to_parent(result: &mut Result, new_node: MenuItem, parent_id: usize) {
    if let Some(path_to_parent) = result.nodes_map.get(&parent_id) {
        let mut walked_node: &mut MenuItem = &mut result.nodes[path_to_parent[0]];

        for i in 1..path_to_parent.len() {
            walked_node = &mut walked_node.children[path_to_parent[i]];
        }

        let new_id = new_node.id.clone();
        walked_node.children.push(new_node);
        let mut new_path = path_to_parent.clone();
        new_path.push(walked_node.children.len() - 1);
        result.nodes_map.insert(new_id, new_path);
    }
}
