use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ProgressPayload {
    pub value: u32,
    pub total: u32,
}
impl ProgressPayload {
    pub fn new(value: u32, total: u32) -> Self {
        Self { value, total }
    }
}