#![allow(dead_code)]

use core::panic;
use std::io::Write;

use crate::{
    openapi::{
        common::{OaSchema, RefOr, SchemaType, Type},
        OpenApi,
    },
    utils::AsStatic,
};

trait Definition {
    fn def_typescript(&self) -> String;
}

#[derive(Debug)]
pub struct ApiSchema {
    schemas: Vec<Schema>,
    references: Vec<Reference>,
    apis: Vec<Api>,
}

impl ApiSchema {
    pub fn from_openapi(oa: &OpenApi) -> Self {
        let len = oa.components.schemas.len() + oa.paths.len() * 2;
        let mut aps = Self {
            schemas: Vec::with_capacity(len),
            references: Vec::with_capacity(len),
            apis: Vec::with_capacity(oa.paths.len()),
        };

        for (k, v) in oa.components.schemas.iter() {
            let RefOr::T(s) = v else { panic!("invalid component") };
            let name = k.to_string();

            match s {
                OaSchema::Object(o) => {
                    let mut is_ud = false;
                    if let Some(desc) = &o.description {
                        is_ud = desc.contains("#user_defined") || is_ud;
                    }
                    let SchemaType::Type(ty) = &o.schema_type else {
                        panic!("invalid schema type")
                    };
                    match ty {
                        Type::Object => {
                            let mut _obj = Object {
                                properties: Vec::with_capacity(
                                    o.properties.len(),
                                ),
                            };
                            for (kp, vp) in o.properties.iter() {
                                // match vp {
                                //     RefOr::T(rt) => rt,
                                //     RefOr::Ref(rr) => rr,
                                // }
                                println!("{kp}: {vp:?}");
                                println!("\x1b[34m==================\x1b[0m");
                                // obj.properties.push(Param {name: kp.to_string(), param_in: vp})
                            }
                        }
                        Type::String => {
                            let Some(ev) = &o.enum_values else {
                                aps.schemas.push(Schema::String {});
                                let s = aps.schemas.last().unwrap();
                                aps.references.push(Reference {
                                    name,
                                    schema: s.as_static(),
                                    orphan: false,
                                    is_user_defined: is_ud,
                                });
                                continue;
                            };
                            let x = ev
                                .iter()
                                .map(|v| v.as_str().unwrap().to_string())
                                .collect::<Vec<_>>();
                            aps.schemas.push(Schema::StringUnion(x));
                            let s = aps.schemas.last().unwrap();
                            aps.references.push(Reference {
                                name: k.to_string(),
                                is_user_defined: is_ud,
                                orphan: false,
                                schema: s.as_static(),
                            })
                        }
                        _ => panic!("unknown ty: {ty:?}"),
                    }

                    // o.title;
                    // o.description;
                    // o.content_media_type
                }
                OaSchema::AllOf(a) => {
                    println!("all of \x1b[32m{a:?}\x1b[0m");
                }
                OaSchema::OneOf(o) => {
                    println!("one of \x1b[33m{o:?}\x1b[0m");
                }
                _ => panic!("unknown: {s:?}"),
            }
        }

        aps
    }

    pub fn generate(&self) -> std::io::Result<()> {
        let mut ts = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("argon-data/generated.ts")?;

        ts.write_all("import * as ud from './user_defined';\n\n".as_bytes())?;

        for r in self.references.iter() {
            if r.is_user_defined {
                continue;
            }

            let def = r.schema.def_typescript();
            let exp = format!("export type {} = {def};\n", r.name);
            ts.write_all(exp.as_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Reference {
    name: String,
    is_user_defined: bool,
    orphan: bool,
    schema: &'static Schema,
}

#[derive(Debug)]
enum Schema {
    Object(Object),
    Array { schema: Box<Schema> },
    Union {},
    String {},
    StringUnion(Vec<String>),
    Number {},
    Bool {},
}

impl Definition for Schema {
    fn def_typescript(&self) -> String {
        match self {
            Self::String {} => "string".to_string(),
            Self::StringUnion(su) => su
                .iter()
                .map(|v| format!("'{v}'"))
                .collect::<Vec<_>>()
                .join("|"),
            _ => panic!("no def for {self:?}"),
        }
    }
}

#[derive(Debug)]
struct Object {
    properties: Vec<ObjectProperty>,
}

#[derive(Debug)]
struct ObjectProperty {
    name: String,
    required: bool,
    schema: &'static Reference,
}

#[derive(Debug)]
struct Api {
    name: String,
    params: Vec<Param>,
    url: String,
    body: &'static Reference,
    body_type: String, // application/json, text/plain, ...
    output: &'static Reference,
    output_type: String,
    method: String,
}

#[derive(Debug)]
struct Param {
    name: String,
    param_in: ParamIn,
    required: bool,
    schema: &'static Reference,
}

#[derive(Debug)]
enum ParamIn {
    Path,
    Query,
    Header,
    Cookie,
}
