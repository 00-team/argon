use indexmap::IndexMap;

use crate::models::{
    ApiSchema,
    types::{ApiKind, ApiPrim},
};

// #[derive(Debug)]
pub struct KotlinApi {
    tagged_enums: Vec<TaggedEnum>,
    str_enums: Vec<StrEnum>,
    objects: Vec<Object>,
    typealias: Vec<(String, KotlinPrim)>,
    api_version: String,
}

struct IntermediateApiType {
    kind: ApiKind,
    used: u16,
    is_multipart: bool,
}

impl KotlinApi {
    pub fn new(schema: &ApiSchema) -> Self {
        let kapi = Self {
            typealias: Vec::with_capacity(schema.types.len()),
            objects: Vec::with_capacity(schema.types.len()),
            str_enums: Vec::with_capacity(schema.types.len()),
            tagged_enums: Vec::with_capacity(schema.types.len()),
            api_version: schema.api_version.clone(),
        };
        let mut intermediate =
            IndexMap::<String, IntermediateApiType>::with_capacity(
                schema.types.len(),
            );

        for (name, ty) in schema.types.iter() {
            assert!(!intermediate.contains_key(name));
            assert_eq!(Some(name), ty.name.as_ref());
            intermediate.insert(
                name.clone(),
                IntermediateApiType {
                    kind: ty.kind.clone(),
                    used: 0,
                    is_multipart: false,
                },
            );
        }

        for (_name, ty) in schema.types.iter() {
            match &ty.kind {
                ApiKind::Union(_u) => {}
                ApiKind::Combo(_c) => {}
                ApiKind::Object(_o) => {}
                ApiKind::Prim(p) => {
                    let ApiPrim::Option(o) = p else { continue };
                    let ApiKind::Prim(pp) = &o.kind else { unreachable!() };
                    let ApiPrim::Str = pp else { unreachable!() };
                }
                ApiKind::StrEnum(_) => {}
                _ => unreachable!("{ty:?}"),
            }
        }

        kapi
    }
}

struct Object {
    name: String,
    fields: Vec<ObjectField>,
}

struct StrEnum {
    name: String,
    variants: Vec<String>,
}

struct TaggedEnum {
    tag: String,
    name: String,
    common_props: Vec<ObjectField>,
    variants: Vec<TaggedEnumVariant>,
}

struct TaggedEnumVariant {
    name: String,
    key: String,
    fields: Vec<ObjectField>,
}

struct ObjectField {
    name: String,
    ty: KotlinPrim,
    required: bool,
}

enum KotlinPrim {
    Str,
    Int,
    Float,
    Bool,
    File,
    Ref(String),
    Option(Box<KotlinPrim>),
    Array(Box<KotlinPrim>),
    Map(Box<KotlinPrim>),
}
