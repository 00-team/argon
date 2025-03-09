use super::common::{GetRef, OaSchema, RefOr};
use crate::openapi::common::Def;
use core::panic;
use indoc::formatdoc;
use serde::Deserialize;
use std::collections::HashMap;

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
        &self, url: &str, method: &str, get_ref: &F,
        has_name: &impl Fn(&str) -> bool,
    ) -> (String, String) {
        let name = self.url_to_name(url, method, get_ref, has_name);
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
        let mut query_params = Vec::<&str>::new();

        if let Some(ps) = &self.parameters {
            let mut def = String::from("{");
            params_names.reserve_exact(ps.len());
            for p in ps {
                def.push_str(&p.name);
                params_names.push(&p.name);
                if matches!(p.parameter_in, ParameterIn::Query) {
                    query_params.push(&p.name);
                }
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
            unwrap_params +=
                &format!("let {{ {} }} = params;", params_names.join(","));
        }

        let query_params = query_params.join(",");

        let method_upper = method.to_uppercase();
        let (outy, http_out_type) = self.output_type(get_ref);
        let (body, content_type) = self.body(get_ref);

        let def = formatdoc! {r#"
            /**
            {sum}
            {des}
            */
            export async function {name} ({input}) : Promise<ud.Result<{outy}>> {{
                {unwrap_params}
                {body}
                return new Promise((resolve, reject) => {{
                    ud.httpx({{
                        url: `{ts_url}`,
                        method: "{method_upper}",
                        params: {{ {query_params} }},
                        type: "{http_out_type}",
                        headers: {{
                            'Content-Type': "{content_type}",
                        }},
                        data,
                        reject,
                        onLoad(x) {{
                            resolve({{
                                ok: x.status == 200,
                                status: x.status,
                                body: x.response,
                            }})
                        }}
                    }})
                }})
            }}
        "#,
        };
        (def, name)
    }

    fn body<'a, F: GetRef<'a>>(&self, get_ref: &F) -> (String, &str) {
        let Some(rq) = &self.request_body else {
            return ("let data = undefined;".to_string(), "");
        };
        let (ct, cc) = rq.content.iter().next().unwrap();
        match ct.as_str() {
            "multipart/form-data" => {
                let Some(cs) = &cc.schema else { panic!("no schema") };
                let mut out = String::from("let data = new FormData;\n");
                let cs = match cs {
                    RefOr::T(t) => t,
                    RefOr::Ref(r) => {
                        let RefOr::T(t) = get_ref(r).unwrap().1 else {
                            panic!("nested ref")
                        };
                        t
                    }
                };

                let OaSchema::Object(cs) = cs else { panic!("must be object") };
                for (k, p) in cs.properties.iter() {
                    out.push_str(r#"data.set('"#);
                    out.push_str(k);
                    out.push_str("', ");
                    match p {
                        RefOr::T(_) => {
                            out.push_str("body.");
                            out.push_str(k);
                            // out.push_str(" + ''");
                        }
                        RefOr::Ref(_) => {
                            out.push_str("new Blob([JSON.stringify(");
                            out.push_str("body.");
                            out.push_str(k);
                            out.push_str(")], { type: 'application/json' })");
                        }
                    }
                    out.push_str(");\n");
                }

                // out.push_str(&format!("/*{cs:#?}*/"));

                (out, "multipart/form-data")
            }
            "application/json" => (
                "let data = JSON.stringify(body);".to_string(),
                "application/json",
            ),
            "text/plain" => ("let data = body;".to_string(), "text/plain"),
            _ => panic!("unknown request body: {ct}"),
        }
    }

    fn output_type<'a, F: GetRef<'a>>(&self, get_ref: &F) -> (String, String) {
        let json_ty = String::from("json");
        let Some(RefOr::T(res)) = self.responses.get("200") else {
            return ("any".to_string(), json_ty);
        };
        if res.content.is_empty() {
            return ("void".to_string(), json_ty);
        }
        assert_eq!(res.content.len(), 1);
        let (ct, cc) = res.content.iter().next().unwrap();
        if ct == "text/plain" {
            return ("string".to_string(), "text".to_string());
        }
        assert_eq!(ct, "application/json");
        let Some(cc) = &cc.schema else { return ("any".to_string(), json_ty) };

        (cc.def_ts(get_ref), json_ty)
    }

    fn is_list<'a, F: GetRef<'a>>(&self, get_ref: &F) -> bool {
        let Some(res) = self.responses.get("200") else { return false };
        let RefOr::T(res) = res else { panic!("response is a ref") };
        let Some(c) = res.content.get("application/json") else { return false };
        let Some(c) = &c.schema else { return false };
        let c = match &c {
            RefOr::T(t) => t,
            RefOr::Ref(r) => {
                let Some((_, x)) = get_ref(r) else { return false };
                let RefOr::T(x) = x else { return false };
                x
            }
        };

        let OaSchema::Array(_) = c else { return false };

        true
    }

    fn url_to_name<'a, F: GetRef<'a>>(
        &self, url: &str, method: &str, get_ref: &F,
        _has_name: &impl Fn(&str) -> bool,
    ) -> String {
        let is_list = self.is_list(get_ref);
        let mut name = String::with_capacity(url.len() + method.len() + 10);
        let mut pu = false;
        let mut skip = false;
        let cc = url.chars().count();
        for (i, c) in url.chars().enumerate() {
            if c == '{' {
                skip = true;
            }
            if c == '}' {
                skip = false;
            }
            if skip {
                continue;
            }
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

        if !pu {
            name.push('_');
        }
        if is_list && method == "get" {
            name.push_str("list");
        } else if method == "delete" {
            name.push_str("del");
        } else {
            name.push_str(method);
        }

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
