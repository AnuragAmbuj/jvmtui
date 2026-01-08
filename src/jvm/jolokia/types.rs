use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct JolokiaRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub mbean: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
pub struct JolokiaResponse {
    pub status: u32,
    pub timestamp: u64,
    pub request: Value,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_type: Option<String>,
}

impl JolokiaRequest {
    pub fn read(mbean: &str, attribute: &str) -> Self {
        Self {
            request_type: "read".to_string(),
            mbean: mbean.to_string(),
            attribute: Some(attribute.to_string()),
            operation: None,
            arguments: None,
        }
    }

    pub fn exec(mbean: &str, operation: &str, arguments: Vec<Value>) -> Self {
        Self {
            request_type: "exec".to_string(),
            mbean: mbean.to_string(),
            attribute: None,
            operation: Some(operation.to_string()),
            arguments: Some(arguments),
        }
    }
}
