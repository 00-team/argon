use std::collections::HashMap;

use crate::openapi::common::Def;

use super::common::{GetRef, OaSchema, RefOr};
use indoc::formatdoc;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    // pub summary: Option<String>,
    // pub description: Option<String>,
    // pub servers: Option<Vec<Server>>,
    // pub parameters: Option<Vec<Parameter>>,
    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub patch: Option<Operation>,
    pub delete: Option<Operation>,
    // pub options: Option<Operation>,
    // pub head: Option<Operation>,
    // pub trace: Option<Operation>,
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

impl Operation {
    pub fn def_ts<'a, F: GetRef<'a>>(&self, url: &str, method: &str, get_ref: &F) -> String {
        let name = Self::url_to_name(url, method);
        macro_rules! deopt {
            ($name:ident) => {
                match &self.$name {
                    Some(v) => v.as_str(),
                    None => "",
                }
            };
        }

        let sum = deopt!(summary);
        let des = deopt!(description);
        let mut input = String::from("{");

        if let Some(ps) = &self.parameters {
            let mut def = String::from("{");
            for p in ps {
                def.push_str(&p.name);
                if !p.required {
                    def.push('?');
                }
                def.push(':');
                let Some(s) = &p.schema else {
                    def.push_str("any,");
                    continue;
                };

                let ty = match s {
                    RefOr::T(t) => t.def_ts(get_ref),
                    RefOr::Ref(r) => match get_ref(&r) {
                        Some((i, _)) => i,
                        None => "any".to_string(),
                    },
                };
                def.push_str(&ty);
                def.push(',');
            }
            def.push('}');
            if !ps.is_empty() {
                input.push_str("params:");
                input.push_str(&def);
                input.push(',');
            }
        }

        if let Some(rqb) = &self.request_body {
            // rqb.required;
            // rqb.description;
            // rqb.content;
        }

        input.push('}');
        formatdoc! {"
            /**
            {sum}
            {des}
            */
            export function {name} (input: {input}) : void {{
                /*
                    {:#?}
                    {url}
                */
            }}
        ",
            self.request_body
        }
    }

    fn url_to_name(url: &str, method: &str) -> String {
        let mut name = String::with_capacity(url.len() + method.len() + 10);
        let mut pu = false;
        let cc = url.chars().count();
        for (i, c) in url.chars().enumerate() {
            if matches!(c, '/' | '-' | '_' | '.' | '}' | '{') {
                if pu || i == 0 || i + 1 == cc {
                    continue;
                }
                pu = true;
                name.push('_');
                continue;
            }
            pu = false;
            name.push(c);
        }

        // println!("{name}");

        if !pu {
            name.push('_');
        }
        name.push_str(method);

        name
    }
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
