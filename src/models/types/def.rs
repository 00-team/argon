use std::collections::HashMap;

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

    pub fn dart(&self, for_input: bool) -> String {
        match self {
            ApiPrim::Null => "null".to_string(),
            ApiPrim::Str => "String".to_string(),
            ApiPrim::Int => "int".to_string(),
            ApiPrim::Float => "double".to_string(),
            ApiPrim::File => "http.MultipartFile".to_string(),
            ApiPrim::Bool => "bool".to_string(),
            ApiPrim::Option(opt) => {
                format!("{}?", opt.ref_or_body_dart(for_input))
            }
        }
    }
}

impl ApiType {
    pub fn body_ts(&self, for_input: bool) -> String {
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
                out
            }
            ApiKind::Array(a) => format!("{}[]", a.ref_or_body_ts(for_input)),
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
            ApiKind::Object(o) => {
                let mut inner = String::with_capacity(1024);
                for (p, v) in o {
                    inner.push_str(p);
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
            if !for_input && n == "Gene" {
                return "(Gene | null)".to_string();
            }
            return n.to_string();
        }

        self.body_ts(for_input)
    }
}

// dart

pub fn snake_to_pascal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for w in value.split('_') {
        out.push_str(&w[..1].to_uppercase());
        out.push_str(&w[1..]);
    }
    out
}

impl ApiType {
    pub fn def_dart(&self, for_input: bool) -> String {
        let name = self.name.as_ref().unwrap();
        match &self.kind {
            ApiKind::Unknown => panic!("unknown type: {self:?}"),
            ApiKind::Recursive => unreachable!("recursive"),

            ApiKind::Prim(p) => {
                format!("typedef {name} = {};\n", p.dart(for_input))
            }
            ApiKind::StrEnum(se) => {
                let mut eel = String::with_capacity(512);
                let mut een = String::with_capacity(512);
                for (i, e) in se.iter().enumerate() {
                    let eup = e.to_uppercase();
                    eel.push_str("    static const ");
                    eel.push_str(&format!("{eup} = {name}._('{e}');\n"));
                    een.push_str(&eup);
                    if i + 1 != se.len() {
                        een.push(',');
                    }
                }

                formatdoc! {"
                    class {name} {{
                        final String value;
                        const {name}._(this.value);
                        
                    {eel}
                        
                        static const values = [{een}];

                        factory {name}.fromJson(String json) {{
                            return values.firstWhere((v) => v.value == json);
                        }}

                        String toJson() => value;
                    }}
                "}
            }
            ApiKind::Array(a) => format!(
                "typedef {name} = List<{}>;",
                a.ref_or_body_dart(for_input)
            ),
            ApiKind::Object(o) => Self::dart_object(name, o, for_input),

            ApiKind::Combo(c) => {
                let mut all = ApiObject::with_capacity(64);
                let mut uni: Option<&ApiUnion> = None;
                for x in c {
                    match &x.kind {
                        ApiKind::Union(u) => uni = Some(u),
                        ApiKind::Object(o) => all.extend_from_slice(o),
                        _ => unreachable!(),
                    }
                }

                if let Some(uni) = uni {
                    Self::dart_union(name, uni, for_input, &all)
                } else {
                    Self::dart_object(name, &all, for_input)
                }
            }
            ApiKind::Union(uni) => {
                Self::dart_union(name, uni, for_input, &vec![])
            }
            ApiKind::Ref(r) => format!("typedef {name} = {r};\n"),
            ApiKind::Tuple(tup) => format!("// Tuple: {name} = {tup:?}"),
        }
    }

    pub fn ref_or_body_dart(&self, for_input: bool) -> String {
        if let Some(n) = &self.name {
            return n.to_string();
        }

        match &self.kind {
            ApiKind::Prim(p) => p.dart(for_input),
            ApiKind::Object(obj) => {
                let mut inner = String::with_capacity(1024);
                for (p, v) in obj {
                    inner.push_str(&v.ref_or_body_dart(for_input));
                    inner.push(' ');
                    inner.push_str(p);
                    inner.push(',');
                }
                format!("({{ {inner} }})")
            }
            ApiKind::Array(at) => {
                format!("List<{}>", at.ref_or_body_dart(for_input))
            }
            _ => unreachable!("{self:#?}"),
        }
    }

    fn dart_union(
        name: &str, uni: &ApiUnion, for_input: bool, added: &ApiObject,
    ) -> String {
        let mut ukeys = HashMap::<String, usize>::with_capacity(2);
        // let mut init = String::with_capacity(1024);
        // let mut from_json = String::with_capacity(1024);
        // let mut into_json = String::with_capacity(1024);

        let mut find_uk = |obj: &Vec<(String, ApiType)>| {
            for (n, v) in obj {
                if let ApiKind::StrEnum(se) = &v.kind {
                    if se.len() == 1 {
                        if let Some(uk) = ukeys.get_mut(n) {
                            *uk += 1;
                        } else {
                            ukeys.insert(n.to_string(), 1);
                        }
                    }
                }
            }
        };

        for u in uni {
            match &u.kind {
                ApiKind::Object(obj) => find_uk(obj),
                ApiKind::Combo(c) => {
                    for a in c {
                        if let ApiKind::Object(obj) = &a.kind {
                            find_uk(obj);
                            continue;
                        }
                        unreachable!()
                    }
                }
                _ => unreachable!(),
            }
        }

        let (uk, _) = ukeys.iter().max_by_key(|(_, t)| **t).unwrap();

        let mut vars = String::with_capacity(4096);

        for u in uni {
            let mut inner = String::with_capacity(512);
            let mut var_key = String::new();

            let mut dooj = |obj: &Vec<(String, ApiType)>| {
                for (k, v) in obj {
                    if k == uk {
                        let ApiKind::StrEnum(se) = &v.kind else {
                            unreachable!()
                        };
                        var_key = snake_to_pascal(&se[0]);
                        continue;
                    }

                    inner += &format!(
                        "required {} {k},\n",
                        v.ref_or_body_dart(for_input)
                    );
                }
            };

            dooj(added);

            match &u.kind {
                ApiKind::Object(o) => {
                    dooj(o);
                }
                ApiKind::Combo(c) => {
                    for at in c {
                        let ApiKind::Object(o) = &at.kind else {
                            unreachable!()
                        };
                        dooj(o);
                    }
                }
                _ => panic!("union is not object or combo"),
            }

            if inner.is_empty() {
                vars += &format!(
                    "const factory {name}.{var_key}() = {name}{var_key};\n"
                );
            } else {
                vars += &format!("const factory {name}.{var_key}({{{inner}}}) = {name}{var_key};\n");
            }
        }

        formatdoc! {"
            @Freezed(unionKey: '{uk}', unionValueCase: FreezedUnionCase.snake)
            class {name} with _${name} {{
                {vars}

                factory {name}.fromJson(JsonObject json) => _${name}FromJson(json);
            }}
        "}
    }

    fn dart_object(name: &str, object: &ApiObject, for_input: bool) -> String {
        let gg = object.iter().any(|(k, _)| k.starts_with('_'));
        if !gg {
            let mut inner = String::with_capacity(2048);
            for (p, v) in object {
                inner += &format!("{} {p},\n", v.ref_or_body_dart(for_input));
            }
            return format!("typedef {name} = ({{{inner}}});");
        }

        let mut props = String::with_capacity(1024);
        let mut init = String::with_capacity(1024);
        let mut from_json = String::with_capacity(1024);
        let mut into_json = String::with_capacity(1024);

        for (p, v) in object {
            let pn = p.strip_prefix("_").unwrap_or(p);
            props +=
                &format!("final {} {pn};\n", v.ref_or_body_dart(for_input));
            init += &format!("required this.{pn},");
            from_json += &format!("{pn}: json['{p}'],\n");
            into_json += &format!("'{p}': {pn},");
        }
        formatdoc! {"
            class {name} {{
                {props}

                {name}({{ {init} }});

                factory {name}.fromJson(JsonObject json) {{
                    return {name}({from_json});
                }}

                JsonObject toJson() => {{
                    {into_json}
                }};
            }}
        "}
    }
}
