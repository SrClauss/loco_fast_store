use serde::{Deserialize, Serialize};

/// Envelope de resposta padrão da API
/// Todas as respostas seguem este formato para facilitar integração com IA/wizards
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub cursor: Option<String>,
    pub has_more: bool,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            meta: None,
            error: None,
        }
    }

    pub fn paginated(data: T, cursor: Option<String>, has_more: bool, count: usize) -> Self {
        Self {
            ok: true,
            data: Some(data),
            meta: Some(PaginationMeta {
                cursor,
                has_more,
                count,
            }),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            ok: false,
            data: None,
            meta: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            }),
        }
    }

    pub fn error_with_details(code: &str, message: &str, details: serde_json::Value) -> Self {
        Self {
            ok: false,
            data: None,
            meta: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: Some(details),
            }),
        }
    }
}
