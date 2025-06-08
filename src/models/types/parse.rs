use super::*;
use crate::openapi::{
    array::ArrayItems,
    common::{Def, OaSchema, RefOr, SchemaType, Type},
    format::{KnownFormat, SchemaFormat},
};
use std::collections::{HashMap, HashSet};

impl ApiType {
    pub fn parse_openapi(
        name: Option<String>, value: &RefOr<OaSchema>,
        mut parents: HashSet<String>, types: &mut HashMap<String, ApiType>,
        schemas: &HashMap<String, RefOr<OaSchema>>,
    ) -> Self {
        let mut aty = Self::new(ApiKind::Unknown);
        aty.name = name.clone();

        let schema = match value {
            RefOr::T(s) => s,
            RefOr::Ref(r) => {
                let i = r.loc.split('/').last().unwrap();
                if let Some(aty) = types.get(i) {
                    return aty.clone();
                }
                if parents.contains(i) {
                    return ApiType::RECURSIVE.name(i.to_string());
                }
                let Some(s) = schemas.get(i) else {
                    panic!("ref not found: {i}");
                };
                parents.insert(i.to_string());
                let t = ApiType::parse_openapi(
                    Some(i.to_string()),
                    s,
                    parents,
                    types,
                    schemas,
                );
                types.insert(i.to_string(), t.clone());
                return t;
            }
        };

        aty.user_defined = schema.is_user_defined();

        match schema {
            OaSchema::Object(o) => {
                let mut nullable = false;
                let oty = match &o.schema_type {
                    SchemaType::AnyValue => panic!("any: {o:?}"),
                    SchemaType::Array(a) => {
                        assert_eq!(a.len(), 2);
                        nullable = true;
                        &a[0]
                    }
                    SchemaType::Type(t) => t,
                };

                let kind = match oty {
                    Type::Object => {
                        let cap = o.properties.len();
                        let mut obj = Vec::with_capacity(cap);
                        for (kp, vp) in o.properties.iter() {
                            obj.push((
                                kp.to_string(),
                                ApiType::parse_openapi(
                                    None,
                                    vp,
                                    parents.clone(),
                                    types,
                                    schemas,
                                ),
                            ));
                        }
                        ApiKind::Object(obj)
                    }
                    Type::String => 'str: {
                        if let Some(ev) = &o.enum_values {
                            let x = ev
                                .iter()
                                .map(|v| v.as_str().unwrap().to_string())
                                .collect();
                            break 'str ApiKind::StrEnum(x);
                        }

                        if let Some(fmt) = &o.format {
                            if let SchemaFormat::KnownFormat(kf) = fmt {
                                if matches!(kf, KnownFormat::Binary) {
                                    break 'str ApiKind::File;
                                }
                                panic!("unknown str format: {kf:?}");
                            } else {
                                panic!("custom format: {o:#?}");
                            }
                        }

                        ApiKind::Str
                    }
                    Type::Null => ApiKind::Null,
                    Type::Boolean => ApiKind::Bool,
                    Type::Integer => ApiKind::Int,
                    Type::Number => ApiKind::Int,
                    _ => panic!("unknown oty: {oty:?}"),
                };

                if nullable {
                    let ak = ApiType::new(kind);
                    aty.kind = ApiKind::Union(vec![ak, ApiType::NULL]);
                } else {
                    aty.kind = kind;
                }

                // o.title;
                // o.description;
                // o.content_media_type
            }
            OaSchema::AllOf(af) => {
                aty.kind = ApiKind::Combo(
                    af.items
                        .iter()
                        .map(|v| {
                            ApiType::parse_openapi(
                                None,
                                v,
                                parents.clone(),
                                types,
                                schemas,
                            )
                        })
                        .collect(),
                );
            }
            OaSchema::OneOf(of) => {
                let mut uni = Vec::with_capacity(of.items.len());
                for v in &of.items {
                    uni.push(ApiType::parse_openapi(
                        None,
                        v,
                        parents.clone(),
                        types,
                        schemas,
                    ));
                }
                aty.kind = ApiKind::Union(uni);
            }
            OaSchema::Array(a) => {
                let ArrayItems::R(item) = &a.items else {
                    assert!(!a.prefix_items.is_empty());
                    aty.kind = ApiKind::Tuple(
                        a.prefix_items
                            .iter()
                            .map(|v| {
                                ApiType::parse_openapi(
                                    None,
                                    &RefOr::T(v.clone()),
                                    parents.clone(),
                                    types,
                                    schemas,
                                )
                            })
                            .collect(),
                    );

                    return aty;
                };

                let item =
                    ApiType::parse_openapi(None, item, parents, types, schemas);

                // if a.max_items == a.min_items && a.max_items.is_some() {
                //     let len = a.max_items.unwrap();
                //     println!("len: {len}");
                //     // aty.kind = ApiKind::Tuple(vec![item; len]);
                //     return aty;
                // }

                aty.kind = ApiKind::Array(Box::new(item));

                // a.items;
                // a.prefix_items;
                // a.schema_type;
                // a.max_items;
                // a.min_items;
            }
            s => panic!("unknown: {s:#?}"),
        };

        if matches!(aty.kind, ApiKind::Unknown) {
            panic!("returing unknown: {value:#?}");
        }

        aty
    }
}
