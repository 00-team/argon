use std::collections::HashMap;

use crate::{
    models::types::ApiKind,
    openapi::{
        common::{OaSchema, RefOr},
        path::{Operation, RequestBody, Response},
    },
};

use super::*;

impl ApiRoute {
    pub fn parse_openapi(
        url: &str,
        method: &str,
        op: &Operation,
        types: &mut HashMap<String, ApiType>,
        schemas: &HashMap<String, RefOr<OaSchema>>,
        // routes: &HashMap<String, ApiRoute>,
    ) -> Self {
        // op.responses;
        // op.request_body;
        // op.operation_id;
        // op.parameters;
        // op.description;
        // op.deprecated;
        // op.summary;
        // op.tags;

        let res = match op.responses.get("200") {
            Some(RefOr::T(t)) => Some(t),
            _ => None,
        };

        // println!("url: [{method}] {url}");
        let rb = ApiResponseBody::parse_openapi(res, types, schemas);
        let is_list = 'a: {
            let Some(rb) = &rb else { break 'a false };
            let Some(ty) = &rb.api_type else { break 'a false };
            matches!(ty.kind, ApiKind::Array(_))
        };

        let mut params = Vec::with_capacity(10);
        if let Some(prs) = &op.parameters {
            for p in prs {
                let Some(psh) = &p.schema else {
                    panic!("param with no schema: {op:#?}");
                };

                params.push(ApiParam {
                    name: p.name.to_string(),
                    required: p.required,
                    param_in: p.parameter_in.into(),
                    api_type: ApiType::parse_openapi(
                        None,
                        psh,
                        Default::default(),
                        types,
                        schemas,
                    ),
                });
            }
        }

        let ar = Self {
            request_body: ApiRequstBody::parse_openapi(
                &op.request_body,
                types,
                schemas,
            ),
            url: url.to_string(),
            name: op.url_to_name(url, method, is_list),
            params,
            method: method.to_string(),
            response_body: rb,
            doc: format!(
                "{}\n{}",
                op.summary.as_ref().map(|v| v.as_str()).unwrap_or_default(),
                op.description.as_ref().map(|v| v.as_str()).unwrap_or_default(),
            ),
        };

        ar
    }
}

impl ApiRequstBody {
    pub fn parse_openapi(
        rb: &Option<RequestBody>, types: &mut HashMap<String, ApiType>,
        schemas: &HashMap<String, RefOr<OaSchema>>,
    ) -> Option<Self> {
        let Some(rb) = rb else { return None };
        assert!(rb.required.unwrap_or_default());
        assert!(!rb.content.is_empty());
        let (ct, c) = rb.content.iter().next().unwrap();

        let cs = c.schema.as_ref().unwrap();

        Some(Self {
            content_type: ct.to_string(),
            api_type: ApiType::parse_openapi(
                None,
                cs,
                Default::default(),
                types,
                schemas,
            ),
        })
    }
}

impl ApiResponseBody {
    pub fn parse_openapi(
        res: Option<&Response>, types: &mut HashMap<String, ApiType>,
        schemas: &HashMap<String, RefOr<OaSchema>>,
    ) -> Option<Self> {
        let Some(r) = res else { return None };
        if r.content.is_empty() {
            return None;
        }
        let (ct, c) = r.content.iter().next().unwrap();

        Some(Self {
            content_type: ct.to_string(),
            api_type: c.schema.as_ref().map(|v| {
                ApiType::parse_openapi(
                    None,
                    v,
                    Default::default(),
                    types,
                    schemas,
                )
            }),
        })
    }
}
