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
    pub fn def_ts<'a, F: GetRef<'a>>(
        &self,
        url: &str,
        method: &str,
        get_ref: &F,
        has_name: &impl Fn(&str) -> bool,
    ) -> (String, String) {
        let name = Self::url_to_name(url, method, has_name);
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
        let mut input = Vec::<String>::with_capacity(2);
        let mut params_names = Vec::<&str>::new();

        if let Some(ps) = &self.parameters {
            let mut def = String::from("{");
            params_names.reserve_exact(ps.len());
            for p in ps {
                def.push_str(&p.name);
                params_names.push(&p.name);
                if !p.required {
                    assert!(false);
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
                input.push(format!("params: {def}"))
            }
        }

        if let Some(rq) = &self.request_body {
            let mut body = String::from("body");
            if !rq.required.unwrap_or(false) {
                body.push('?');
            }
            body.push(':');
            body.push_str(&rq.def_ts(get_ref));
            input.push(body);
        }

        let input = input.join(",");
        let ts_url = url.replace('{', "${");
        let mut unwrap_params = String::new();
        if !params_names.is_empty() {
            unwrap_params += &format!("let {{ {} }} = params;", params_names.join(","));
        }

        let method_upper = method.to_uppercase();

        let def = formatdoc! {r#"
            /**
            {sum}
            {des}
            */
            export async function {name} ({input}) : Promise<void> {{
                {unwrap_params}
                new Promise((resolve, reject) => {{
                    ud.httpx({{
                        url: `{ts_url}`,
                        method: "{method_upper}",
                        reject,
                    }})
                }})
                /*
                    {url}
                */
            }}
        "#,
        };
        (def, name)
    }

   fn url_to_name(url: &str, method: &str, has_name: &impl Fn(&str) -> bool) -> String {
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

impl Def for RequestBody {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        assert!(self.required.is_some());
        assert_eq!(self.content.len(), 1);
        let (ct, c) = self.content.iter().next().unwrap();
        let Some(cs) = &c.schema else {
            return "any".to_string();
        };
        match ct.as_str() {
            "application/json" | "multipart/form-data" => {
                match cs {
                    RefOr::Ref(r) => match get_ref(r) {
                        Some((i, _)) => i,
                        None => "any".to_string(),
                    },
                    RefOr::T(t) => t.def_ts(get_ref),
                }
                // println!("{c:#?}");
            }
            "text/plain" => "string".to_string(),
            _ => panic!("unknown request body content: {ct}"),
        }
    }
}
