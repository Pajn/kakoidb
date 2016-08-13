pub fn root_key() -> String {
    "root".to_string()
}

pub fn list_key(id: &str) -> String {
    format!("list_{}", id)
}

pub fn node_key(id: &str) -> String {
    format!("node_{}", id)
}
