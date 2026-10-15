use indoc::formatdoc;

use crate::utils::{default_name, pascal_to_snake};

use super::*;

impl ApiPrim {
    pub fn ts(&self, for_input: bool) -> String {
        match self {
            ApiPrim::Null => "null".to_string(),
            ApiPrim::Str => "string".to_string(),
            ApiPrim::Int => "number".to_string(),
            ApiPrim::Float => "number".to_string(),
            ApiPrim::File => "File".to_string(),
            ApiPrim::Bool => "boolean".to_string(),
            ApiPrim::Option(opt) => {
                format!("({} | null)", opt.ref_or_body_ts(for_input))
            }
        }
    }

    pub fn ts_default(&self) -> &'static str {
        match self {
            ApiPrim::Null => "null",
            ApiPrim::Str => "''",
            ApiPrim::Int => "0",
            ApiPrim::Float => "0",
            ApiPrim::File => "new File([], '')",
            ApiPrim::Bool => "false",
            ApiPrim::Option(_) => "null",
        }
    }
}

impl ApiType {
    pub fn body_ts(&self, def: bool, for_input: bool) -> String {
        match &self.kind {
            ApiKind::Prim(p) => p.ts(for_input),
            ApiKind::Unknown => panic!("unknown type: {self:?}"),
            ApiKind::Ref(r) => r.to_string(),
            ApiKind::Recursive => unreachable!("recursive"),
            ApiKind::StrEnum(e) => {
                let mut out = String::with_capacity(1024);
                let len = e.len();
                for (i, se) in e.iter().enumerate() {
                    out.push('"');
                    out.push_str(se);
                    out.push('"');
                    if i + 1 != len {
                        out.push('|');
                    }
                }

                if def && let Some(name) = &self.name {
                    out.push_str(";\n");
                    let snake = pascal_to_snake(name);
                    let cname = snake.to_uppercase();
                    out.push_str("export const ");
                    out.push_str(&cname);
                    out.push_str(" = [");
                    for se in e.iter() {
                        out.push('"');
                        out.push_str(se);
                        out.push_str(r#"","#);
                    }
                    out.push_str("] as const;\n");
                    let lname = snake.to_lowercase();
                    out += &formatdoc! {"
                    export function is_{lname}(value: string): value is {name} {{
                        return {cname}.includes(value as {name})
                    }}
                    "};
                }

                out
            }
            ApiKind::Array(a) => format!("({}[])", a.ref_or_body_ts(for_input)),
            ApiKind::Union(u) => u
                .iter()
                .map(|v| v.ref_or_body_ts(for_input))
                .collect::<Vec<_>>()
                .join("|"),
            ApiKind::Combo(c) => c
                .iter()
                .map(|v| v.ref_or_body_ts(for_input))
                .collect::<Vec<_>>()
                .join("&"),
            ApiKind::Map(val) => {
                format!("_Map<{}>", val.ref_or_body_ts(for_input))
            }
            ApiKind::Object(o) => {
                let mut inner = String::with_capacity(1024);
                for (p, v, rq) in o {
                    inner.push_str(p);
                    if !rq {
                        inner.push('?');
                    }
                    inner.push(':');
                    inner.push_str(&v.ref_or_body_ts(for_input));
                    inner.push(',');
                }
                format!("{{ {inner} }}")
            }
            ApiKind::Tuple(t) => {
                let mut inner = String::with_capacity(1024);
                for v in t {
                    inner.push_str(&v.ref_or_body_ts(for_input));
                    inner.push(',');
                }
                format!("[ {inner} ]")
            }
        }
    }

    pub fn ref_or_body_ts(&self, for_input: bool) -> String {
        if let Some(n) = &self.name {
            // if !for_input && n == "Gene" {
            //     return "(Gene | null)".to_string();
            // }
            return n.to_string();
        }

        format!("({})", self.body_ts(false, for_input))
    }

    pub fn body_ts_default(&self) -> String {
        match &self.kind {
            ApiKind::Prim(p) => p.ts_default().to_string(),
            ApiKind::Unknown => panic!("unknown type: {self:?}"),
            ApiKind::Ref(r) => default_name(r),
            ApiKind::Recursive => unreachable!("recursive"),
            ApiKind::StrEnum(e) => {
                format!("'{}'", e.first().unwrap())
            }
            ApiKind::Array(_) => "[]".to_string(),
            ApiKind::Union(u) => u.first().unwrap().default_ref_or_body_ts(),
            ApiKind::Combo(c) => {
                let mut out = String::with_capacity(512);
                out.push('{');

                for aty in c {
                    out.push_str("...");
                    out += &aty.default_ref_or_body_ts();
                    out.push(',');
                }

                out.push('}');
                out
            }
            ApiKind::Map(_) => "{}".to_string(),
            ApiKind::Object(o) => {
                let mut inner = String::with_capacity(1024);
                inner.push('{');
                for (p, v, rq) in o {
                    if !rq {
                        continue;
                    }
                    inner.push_str(p);
                    inner.push(':');
                    inner.push_str(&v.default_ref_or_body_ts());
                    inner.push(',');
                }
                inner.push('}');
                inner
            }
            ApiKind::Tuple(t) => {
                let mut inner = String::with_capacity(1024);
                inner.push('[');
                for v in t {
                    inner.push_str(&v.default_ref_or_body_ts());
                    inner.push(',');
                }
                inner.push(']');
                inner
            }
        }
    }

    pub fn default_ref_or_body_ts(&self) -> String {
        if let Some(n) = &self.name {
            return default_name(n);
        }

        format!("({})", self.body_ts_default())
    }
}
