use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub status: u16,
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResult<T> {
    pub success: bool,
    pub status: u16,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResult<T> {
    pub fn success(status: u16, data: T) -> Self {
        Self {
            success: true,
            status,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure_localized<S>(
        status: u16,
        code: Option<S>,
        message: impl Into<String>,
        params: Option<Value>,
        details: Option<Value>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            success: false,
            status,
            data: None,
            error: Some(ApiError {
                status,
                message: message.into(),
                code: code.map(Into::into),
                params,
                details,
            }),
        }
    }
}
