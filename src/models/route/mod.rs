use crate::openapi::path::ParameterIn;

use super::types::ApiType;

mod def_ts;
mod def_dart;
mod parse;

#[derive(Debug, Clone)]
pub struct ApiRoute {
    pub doc: String,
    pub name: String,
    pub params: Vec<ApiParam>,
    pub url: String,
    pub request_body: Option<ApiRequstBody>,
    pub response_body: Option<ApiResponseBody>,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct ApiParam {
    pub name: String,
    pub param_in: ApiParamIn,
    pub required: bool,
    pub api_type: ApiType,
}

#[derive(Debug, Clone)]
pub enum ApiParamIn {
    Path,
    Query,
    Header,
    Cookie,
}

impl ApiParamIn {
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path)
    }

    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header)
    }

    pub fn is_cookie(&self) -> bool {
        matches!(self, Self::Cookie)
    }
}

impl From<ParameterIn> for ApiParamIn {
    fn from(value: ParameterIn) -> Self {
        match value {
            ParameterIn::Path => Self::Path,
            ParameterIn::Query => Self::Query,
            ParameterIn::Header => Self::Header,
            ParameterIn::Cookie => Self::Cookie,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiResponseBody {
    pub content_type: String,
    pub api_type: Option<ApiType>,
}

#[derive(Debug, Clone)]
pub struct ApiRequstBody {
    pub content_type: String,
    pub api_type: ApiType,
}
