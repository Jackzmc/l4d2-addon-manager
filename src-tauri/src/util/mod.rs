use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SetRoute {
    pub name: Option<String>,
}