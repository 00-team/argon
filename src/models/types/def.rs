use indoc::formatdoc;

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

    pub fn ts_def(&self) -> &'static str {
        match self {
            ApiPrim::Null => "null",
            ApiPrim::Str => "''",
            ApiPrim::Int => "0",
            ApiPrim::Float => "0",
            ApiPrim::File => "new File()",
            ApiPrim::Bool => "false",
            ApiPrim::Option(_) => "null",
        }
    }
}

pub fn pascal_to_snake(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 2);
    for (i, ch) in value.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            out.push('_');
        }
        out.push(ch);
    }

    if let Some(out) = out.strip_suffix('_') { out.to_string() } else { out }
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

                if def && self.name.is_some() {
                    out.push_str(";\n");
                    let name = self.name.as_ref().unwrap();
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
}
