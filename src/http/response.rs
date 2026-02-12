use serde::{Deserialize, Serialize};

use crate::types::common::Pagination;
use crate::types::enums::{ResponseStatus, StreamDataType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedApiResponse<T> {
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedStreamResponse<T> {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub data_type: Option<StreamDataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub ts: i64,
    pub seq: i64,
}
