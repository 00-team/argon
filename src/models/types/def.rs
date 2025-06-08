use super::*;

impl ApiType {
    pub fn body(&self, for_input: bool) -> String {
        match &self.kind {
            ApiKind::Null => "null".to_string(),
            ApiKind::Str => "string".to_string(),
            ApiKind::Int => "number".to_string(),
            ApiKind::File => "File".to_string(),
            ApiKind::Bool => "boolean".to_string(),
            ApiKind::Unknown => panic!("unknown type: {self:?}"),
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
                out
            }
            ApiKind::Array(a) => format!("{}[]", a.ref_or_body(for_input)),
            ApiKind::Union(u) => {
                u.iter().map(|v| v.ref_or_body(for_input)).collect::<Vec<_>>().join("|")
            }
            ApiKind::Combo(c) => {
                c.iter().map(|v| v.ref_or_body(for_input)).collect::<Vec<_>>().join("&")
            }
            ApiKind::Object(o) => {
                let mut inner = String::with_capacity(1024);
                for (p, v) in o {
                    inner.push_str(p);
                    inner.push(':');
                    inner.push_str(&v.ref_or_body(for_input));
                    inner.push(',');
                }
                format!("{{ {inner} }}")
            }
            ApiKind::Tuple(t) => {
                let mut inner = String::with_capacity(1024);
                for v in t {
                    inner.push_str(&v.ref_or_body(for_input));
                    inner.push(',');
                }
                format!("[ {inner} ]")
            }
        }
    }

    pub fn ref_or_body(&self, for_input: bool) -> String {
        if let Some(n) = &self.name {
            if !for_input && n == "Gene" {
                return "Gene | null".to_string();
            }
            return n.to_string();
        }

        self.body(for_input)
    }
}
