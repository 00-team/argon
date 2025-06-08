use indoc::formatdoc;

use crate::models::types::ApiKind;

use super::*;

impl ApiRoute {
    pub fn def_typescript(&self) -> String {
        let (outy, http_out_type) = match &self.response_body {
            Some(ab) => match ab.content_type.as_str() {
                "text/plain" => ("string".to_string(), "type: 'text',"),
                "application/octet-stream" => {
                    ("ArrayBuffer".to_string(), "type: 'arraybuffer',")
                }
                "application/json" => {
                    let Some(ty) = &ab.api_type else {
                        panic!("json response body is none: {self:#?}");
                    };

                    (ty.ref_or_body(false), "type: 'json',")
                }
                _ => panic!("unknown response type: {self:#?}"),
            },
            None => ("void".to_string(), ""),
        };

        let mut input = Vec::<String>::with_capacity(10);
        let mut query_params = Vec::with_capacity(10);
        let mut bloom_names = Vec::with_capacity(10);

        if !self.params.is_empty() {
            let mut pi = String::with_capacity(512);
            pi.push_str("params: {");

            for p in self.params.iter() {
                assert!(p.required);
                bloom_names.push(p.name.as_str());

                if p.param_in.is_query() {
                    query_params.push(p.name.as_str());
                }

                pi.push_str(&p.name);
                pi.push(':');
                pi.push_str(&p.api_type.ref_or_body(true));
                pi.push(',');
            }

            pi.push('}');
            input.push(pi);
        }

        let mut headers = String::with_capacity(512);
        let mut body = String::with_capacity(1024);
        body.push_str("let data = {};");

        if let Some(rb) = &self.request_body {
            input.push(format!("body: {}", rb.api_type.ref_or_body(true)));

            if rb.content_type != "multipart/form-data" {
                headers.push_str("'Content-Type': '");
                headers.push_str(&rb.content_type);
                headers.push_str("',");
            }

            body.clear();
            match rb.content_type.as_str() {
                "text/plain" => {
                    body.push_str("let data = body;");
                }
                "application/json" => {
                    body.push_str("let data = JSON.stringify(body);");
                }
                "multipart/form-data" => {
                    body.push_str("let data = new FormData();\n");
                    let ApiKind::Object(obj) = &rb.api_type.kind else {
                        panic!("multipart body must be an object");
                    };

                    for (name, ty) in obj {
                        if ty.is_nullable() {
                            body.push_str(&format!("body.{name} && "));
                        }

                        if matches!(ty.kind, ApiKind::Str | ApiKind::File) {
                            body.push_str("data.set('");
                            body.push_str(name);
                            body.push_str("', body.");
                            body.push_str(name);
                            body.push_str(");\n");
                        } else {
                            body.push_str(&formatdoc! {"
                                data.set(
                                    '{name}',
                                    new Blob(
                                        [JSON.stringify(body.{name})],
                                        {{ type: 'application/json' }}
                                    )
                                );\n
                            "});
                        }
                    }
                }
                _ => panic!("unknown request_body: {self:#?}"),
            }
        }

        input.push("override: Partial<ud.HttpxProps> = {}".to_string());

        let input = input.join(", ");
        let ts_url = self.url.replace('{', "${");

        let params_bloom = if !bloom_names.is_empty() {
            format!("let {{ {} }} = params;", bloom_names.join(","))
        } else {
            String::new()
        };
        let query_params = query_params.join(",");
        let method_upper = self.method.to_uppercase();

        formatdoc! {r#"
            /**
            {doc}
            */
            export async function {} ({input}) : Promise<ud.Result<{outy}>> {{
                {params_bloom}
                {body}
                return new Promise((resolve, reject) => {{
                    ud.httpx({{
                        url: `{ts_url}`,
                        method: '{method_upper}',
                        params: {{ {query_params} }},
                        {http_out_type}
                        headers: {{ {headers} }},
                        data,
                        reject,
                        onLoad(x) {{
                            resolve({{
                                x,
                                status: x.status,
                                body: x.response,
                                ok(): this is ud.Ok<{outy}> {{
                                    return this.status == 200
                                }},
                                err(): this is ud.Err {{
                                    return !this.ok()
                                }},
                            }})
                        }},
                        ...override
                    }})
                }})

            }}
        "#,
            self.name,
            doc = self.doc,
        }
    }
}
