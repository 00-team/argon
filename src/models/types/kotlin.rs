use crate::models::types::def::snake_to_pascal;

use super::*;

impl ApiType {
    pub fn body_kotlin(&self, def: bool, for_input: bool) -> String {
        match &self.kind {
            // ApiKind::Prim(p) => {
            //     format!(
            //         "typealias {} = {}",
            //         self.name.as_ref().unwrap(),
            //         p.kotlin()
            //     )
            // }
            // ApiKind::Unknown => panic!("unknown type: {self:?}"),
            // ApiKind::Ref(r) => r.to_string(),
            // ApiKind::Recursive => unreachable!("recursive"),
            // ApiKind::StrEnum(e) => {
            //     let Some(name) = &self.name else { panic!("no name") };
            //
            //     let mut out = String::with_capacity(1024);
            //     out += &format!("@Serializable\nenum class {name} {{\n");
            //     for item in e {
            //         out += &format!(
            //             r#"    @SerialName("{item}") {},"#,
            //             item.to_uppercase()
            //         );
            //         out.push('\n');
            //     }
            //
            //     out.push('}');
            //     out
            // }
            ApiKind::Array(_) => {
                unreachable!("hi :)");
                // format!(
                //     "typealias {} = List<{}>",
                //     self.name.as_ref().unwrap(),
                //     a.kotlin_ref()
                // )
            }
            ApiKind::Tuple(tp) => {
                // format!("/*\n {tp:#?} \n*/")
                unreachable!()
            }
            // ApiKind::Union(u) => {
            //     let mut out = String::with_capacity(1024);
            //     let name = self.name.as_ref().unwrap();
            //     out += &format!("sealed class {name} {{\n");
            //
            //     let strenum_obj =
            //         u.iter().any(|ty| matches!(ty.kind, ApiKind::StrEnum(_)));
            //
            //     assert!(!strenum_obj);
            //
            //     if strenum_obj {
            //         for ty in u {
            //             assert!(ty.name.is_none());
            //
            //             match &ty.kind {
            //                 ApiKind::StrEnum(k) => {
            //                     assert!(k.len() == 1);
            //                     let k = k.first().unwrap();
            //                     let on = snake_to_pascal(k);
            //                     out += &format!(
            //                         r#"    data object {on} : {name}()"#
            //                     );
            //                     out.push('\n');
            //                 }
            //                 ApiKind::Object(props) => {
            //                     assert!(props.len() == 1);
            //                     let (pn, pty, _) = props.last().unwrap();
            //                     let on = snake_to_pascal(&pn);
            //                     let mut cpr = String::with_capacity(512);
            //                     let ApiKind::Object(ppr) = &pty.kind else {
            //                         unreachable!()
            //                     };
            //                     for (ppn, pty, rq) in ppr {
            //                         cpr.push_str("        ");
            //                         cpr.push_str("val ");
            //                         cpr.push_str(ppn);
            //                         if !rq {
            //                             cpr.push('?');
            //                         }
            //                         cpr.push_str(": ");
            //                         cpr.push_str(&pty.kotlin_ref());
            //                         cpr.push_str(",\n");
            //                     }
            //                     out += &format!(
            //                         "    data class {on}(\n{cpr}) : {name}()"
            //                     );
            //                 }
            //                 _ => unreachable!(),
            //             }
            //         }
            //
            //         out.push('}');
            //         return out;
            //     }

            //     fn key_prop(obj: &ApiObject) -> Option<String> {
            //         for (k, ty, _) in obj {
            //             let ApiKind::StrEnum(st) = &ty.kind else { continue };
            //             assert!(st.len() == 1);
            //             return Some(k.clone());
            //         }
            //
            //         None
            //     }
            //
            //     let Some(key) = u.iter().find_map(|ty| match &ty.kind {
            //         ApiKind::Object(ob) => key_prop(ob),
            //         ApiKind::Combo(co) => {
            //             for t in co {
            //                 if t.name.is_some() {
            //                     continue;
            //                 }
            //                 let ApiKind::Object(ob) = &t.kind else {
            //                     continue;
            //                 };
            //                 if let Some(key) = key_prop(ob) {
            //                     return Some(key);
            //                 }
            //             }
            //             None
            //         }
            //         _ => None,
            //     }) else {
            //         unreachable!()
            //     };
            //
            //     for ty in u {
            //         assert!(ty.name.is_none());
            //
            //         match &ty.kind {
            //             ApiKind::StrEnum(_) => unreachable!(),
            //             ApiKind::Object(props) => {
            //                 let Some(on) =
            //                     props.iter().find_map(|(k, ty, _)| {
            //                         if k != &key {
            //                             return None;
            //                         }
            //                         let ApiKind::StrEnum(se) = &ty.kind else {
            //                             unreachable!()
            //                         };
            //                         return se.first();
            //                     })
            //                 else {
            //                     unreachable!()
            //                 };
            //
            //                 if props.len() == 1 {
            //                     out += &format!(
            //                         "    data object {on}: {name}()\n"
            //                     );
            //                     continue;
            //                 }
            //
            //                 let mut cpr = String::with_capacity(512);
            //                 for (ppn, pty, rq) in props {
            //                     if ppn == &key {
            //                         continue;
            //                     }
            //                     cpr.push_str("        ");
            //                     cpr.push_str("val ");
            //                     cpr.push_str(ppn);
            //                     if !rq {
            //                         cpr.push('?');
            //                     }
            //                     cpr.push_str(": ");
            //                     cpr.push_str(&pty.kotlin_ref());
            //                     cpr.push_str(",\n");
            //                 }
            //                 out += &format!(
            //                     "    data class {on}(\n{cpr}) : {name}()"
            //                 );
            //             }
            //             _ => {} // out += &format!("/* TODO: {ty:#?} */"),
            //         }
            //     }
            //
            //     out.push('}');
            //
            //     return out;
            //
            //     // u
            //     //                   .iter()
            //     //                   .map(|v| v.kotlin_ref(for_input))
            //     //                   .collect::<Vec<_>>()
            //     //                   .join("|")
            // }
            _ => String::new(), // let mut out = String::with_capacity(1024);
                                // let len = e.len();
                                // for (i, se) in e.iter().enumerate() {
                                //     out.push('"');
                                //     out.push_str(se);
                                //     out.push('"');
                                //     if i + 1 != len {
                                //         out.push('|');
                                //     }
                                // }
                                //
                                // if def && self.name.is_some() {
                                //     out.push_str(";\n");
                                //     let name = self.name.as_ref().unwrap();
                                //     let snake = pascal_to_snake(name);
                                //     let cname = snake.to_uppercase();
                                //     out.push_str("export const ");
                                //     out.push_str(&cname);
                                //     out.push_str(" = [");
                                //     for se in e.iter() {
                                //         out.push('"');
                                //         out.push_str(se);
                                //         out.push_str(r#"","#);
                                //     }
                                //     out.push_str("] as const;\n");
                                //     let lname = snake.to_lowercase();
                                //     out += &formatdoc! {"
                                //     export function is_{lname}(value: string): value is {name} {{
                                //         return {cname}.includes(value as {name})
                                //     }}
                                //     "};
                                // }
                                //
                                // out
                                // }

                                // ApiKind::Combo(c) => c
                                //     .iter()
                                //     .map(|v| v.kotlin_ref(for_input))
                                //     .collect::<Vec<_>>()
                                //     .join("&"),
                                // ApiKind::Map(val) => {
                                //     format!("_Map<{}>", val.kotlin_ref(for_input))
                                // }
                                // ApiKind::Object(o) => {
                                //     let mut inner = String::with_capacity(1024);
                                //     for (p, v, rq) in o {
                                //         inner.push_str(p);
                                //         if !rq {
                                //             inner.push('?');
                                //         }
                                //         inner.push(':');
                                //         inner.push_str(&v.kotlin_ref(for_input));
                                //         inner.push(',');
                                //     }
                                //     format!("{{ {inner} }}")
                                // }
                                // ApiKind::Tuple(t) => {
                                //     let mut inner = String::with_capacity(1024);
                                //     for v in t {
                                //         inner.push_str(&v.kotlin_ref(for_input));
                                //         inner.push(',');
                                //     }
                                //     format!("[ {inner} ]")
                                // }
        }
    }

    pub fn kotlin_ref(&self) -> String {
        if let Some(n) = &self.name {
            // if !for_input && n == "Gene" {
            //     return "(Gene | null)".to_string();
            // }
            return n.to_string();
        }

        match &self.kind {
            ApiKind::Prim(p) => p.kotlin(),
            ApiKind::Array(v) => format!("List<{}>", v.kotlin_ref()),
            _ => panic!("no inline allowed in kotlin: {self:#?}"),
        }
    }
}
