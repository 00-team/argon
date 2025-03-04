use std::collections::HashMap;

use super::common::{OaSchema, RefOr};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OaPath {
    get: Option<Operation>,
    put: Option<Operation>,
    post: Option<Operation>,
    delete: Option<Operation>,
    patch: Option<Operation>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    pub summary: Option<String>,
    pub description: Option<String>,
    // pub servers: Option<Vec<Server>>,
    pub parameters: Option<Vec<Parameter>>,
    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub head: Option<Operation>,
    pub patch: Option<Operation>,
    pub trace: Option<Operation>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub parameter_in: ParameterIn,
    pub description: Option<String>,
    pub required: bool,
    pub deprecated: Option<bool>,
    pub schema: Option<RefOr<OaSchema>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    Query,
    #[default]
    Path,
    Header,
    Cookie,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub tags: Option<Vec<String>>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, RefOr<Response>>,
    pub deprecated: Option<bool>,
    // pub security: Option<Vec<SecurityRequirement>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub description: String,
    /// Map of headers identified by their name. `Content-Type` header will be ignored.
    #[serde(default)]
    pub headers: HashMap<String, Header>,
    #[serde(default)]
    pub content: HashMap<String, Content>,
    // #[serde(default)]
    // pub links: BTreeMap<String, RefOr<Link>>,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub schema: RefOr<OaSchema>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Content {
    pub schema: Option<RefOr<OaSchema>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, Content>,
    pub required: Option<bool>,
}
