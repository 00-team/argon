use std::collections::HashMap;

use indexmap::IndexMap;

mod routes;
mod types;

use crate::{
    kotlin::routes::KotlinRoute,
    models::{
        ApiSchema,
        types::{ApiKind, ApiObject, ApiPrim, ApiType, def::snake_to_pascal},
    },
};

const T1: &str = "    ";
const T2: &str = "        ";
const T3: &str = "            ";
const T4: &str = "                ";
const T5: &str = "                    ";
const T6: &str = "                        ";

macro_rules! push {
    ($ident:ident, $($exp:expr),*) => {
        $($ident.push_str($exp);)*
    };
}
pub(self) use push;

// #[derive(Debug)]
pub struct KotlinApi {
    tagged_enums: Vec<TaggedEnum>,
    str_enums: Vec<StrEnum>,
    objects: IndexMap<String, KotlinObject>,
    typealias: Vec<(String, KotlinPrim)>,
    routes: Vec<KotlinRoute>,
    api_version: String,
}

#[derive(Debug)]
struct IntermediateApiType {
    kind: ApiKind,
    used: u16,
    is_multipart: bool,
}

impl KotlinApi {
    pub fn new(schema: &ApiSchema) -> Self {
        let mut kapi = Self {
            typealias: Vec::with_capacity(schema.types.len()),
            objects: IndexMap::with_capacity(schema.types.len()),
            str_enums: Vec::with_capacity(schema.types.len()),
            tagged_enums: Vec::with_capacity(schema.types.len()),
            routes: schema
                .route
                .values()
                .map(|v| KotlinRoute::from(v.clone()))
                .collect(),
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

        fn ref_count(ty: &ApiType, counts: &mut HashMap<String, u16>) {
            if let Some(name) = &ty.name {
                if let Some(cc) = counts.get_mut(name) {
                    *cc += 1;
                    return;
                } else {
                    counts.insert(name.clone(), 0);
                }
            }

            match &ty.kind {
                ApiKind::Ref(name) => {
                    if let Some(cc) = counts.get_mut(name) {
                        *cc += 1;
                    } else {
                        counts.insert(name.clone(), 1);
                    }
                }
                ApiKind::Prim(p) => {
                    let ApiPrim::Option(o) = p else { return };
                    ref_count(o, counts);
                }
                ApiKind::Array(t) => ref_count(t, counts),
                ApiKind::Map(t) => ref_count(t, counts),
                ApiKind::Object(o) => {
                    o.iter().for_each(|(_, t, _)| ref_count(t, counts));
                }
                ApiKind::Combo(c) => {
                    c.iter().for_each(|t| ref_count(t, counts))
                }
                ApiKind::Union(u) => {
                    u.iter().for_each(|t| ref_count(t, counts))
                }
                ApiKind::Tuple(t) => {
                    t.iter().for_each(|t| ref_count(t, counts))
                }
                _ => {}
            }
        }

        let mut counts = HashMap::with_capacity(intermediate.len());

        for (_, ty) in schema.types.iter() {
            ref_count(ty, &mut counts);
        }

        for (_, r) in schema.route.iter() {
            r.params.iter().for_each(|p| ref_count(&p.api_type, &mut counts));
            if let Some(res) = &r.response_body {
                let ismp = res.content_type == "multipart/form-data";
                assert!(!ismp);
                if let Some(ty) = &res.api_type {
                    ref_count(ty, &mut counts);
                }
            }

            if let Some(req) = &r.request_body {
                let ismp = req.content_type == "multipart/form-data";
                if ismp {
                    let it = intermediate
                        .get_mut(req.api_type.name.as_ref().unwrap())
                        .unwrap();
                    it.is_multipart = true;
                }
                ref_count(&req.api_type, &mut counts);
            }
        }

        for (name, count) in counts.iter() {
            let it = intermediate.get_mut(name).unwrap();
            it.used = *count;
        }

        for (name, it) in intermediate.iter() {
            // println!("{name}");
            match &it.kind {
                ApiKind::Combo(co) => {
                    assert!(co.len() == 2, "{name}");

                    let (a, b) = (&co[0], &co[1]);
                    let ApiKind::Union(auni) = &a.kind else { unreachable!() };
                    let ApiKind::Object(bob) = &b.kind else { unreachable!() };

                    let mut te = TaggedEnum::from_union(name, auni);
                    for (k, ty, rq) in bob {
                        te.common_props.push(ObjectField {
                            name: k.clone(),
                            ty: KotlinPrim::from_aty(ty),
                            required: *rq,
                        });
                    }
                    kapi.tagged_enums.push(te);
                }
                ApiKind::Union(uni) => {
                    let te = TaggedEnum::from_union(name, uni);
                    kapi.tagged_enums.push(te);
                }
                ApiKind::Ref(rr) => {
                    kapi.typealias
                        .push((name.clone(), KotlinPrim::Ref(rr.clone())));
                }
                ApiKind::Object(obj) => {
                    let kob = KotlinObject::from_fields(
                        name.clone(),
                        obj,
                        it.is_multipart,
                    );
                    kapi.objects.insert(kob.name.clone(), kob);
                }
                ApiKind::StrEnum(se) => {
                    let en =
                        StrEnum { name: name.clone(), variants: se.clone() };
                    kapi.str_enums.push(en);
                }
                ApiKind::Prim(p) => {
                    kapi.typealias
                        .push((name.clone(), KotlinPrim::from_aprim(p)));
                }
                _ => unreachable!("{it:?}"),
            }
        }

        kapi
    }
}

#[derive(Debug, Clone)]
struct KotlinObject {
    name: String,
    fields: Vec<ObjectField>,
    is_multipart: bool,
}

impl KotlinObject {
    pub fn from_fields(
        name: String, fields: &[(String, ApiType, bool)], is_multipart: bool,
    ) -> Self {
        let mut kob = KotlinObject {
            name,
            fields: Vec::with_capacity(fields.len()),
            is_multipart,
        };
        for (k, ty, rq) in fields {
            kob.fields.push(ObjectField {
                name: k.clone(),
                ty: KotlinPrim::from_aty(ty),
                required: *rq,
            });
        }

        kob
    }
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

impl TaggedEnum {
    pub fn from_union(name: &String, union: &[ApiType]) -> Self {
        fn key_prop(obj: &ApiObject) -> Option<String> {
            for (k, ty, _) in obj {
                let ApiKind::StrEnum(st) = &ty.kind else { continue };
                assert!(st.len() == 1);
                return Some(k.clone());
            }

            None
        }

        let tag = union
            .iter()
            .find_map(|ty| match &ty.kind {
                ApiKind::Object(ob) => key_prop(ob),
                ApiKind::Combo(co) => {
                    for t in co {
                        if t.name.is_some() {
                            continue;
                        }
                        let ApiKind::Object(ob) = &t.kind else { continue };
                        let Some(key) = key_prop(ob) else { continue };
                        return Some(key);
                    }
                    None
                }
                _ => None,
            })
            .unwrap();

        let mut te = Self {
            name: name.to_string(),
            tag,
            common_props: Vec::default(),
            variants: Vec::with_capacity(union.len()),
        };

        for item in union {
            let mut var = TaggedEnumVariant::default();

            let fields = match &item.kind {
                ApiKind::Object(obj) => obj.clone(),
                ApiKind::Combo(co) => {
                    assert!(co.len() == 2);
                    let (a, b) = (&co[0], &co[1]);
                    let ApiKind::Object(a) = &a.kind else { unreachable!() };
                    let ApiKind::Object(b) = &b.kind else { unreachable!() };

                    let mut a = a.clone();
                    a.extend(b.iter().cloned());
                    a
                }
                _ => unreachable!(),
            };

            for (k, ty, rq) in &fields {
                if let ApiKind::StrEnum(st) = &ty.kind
                    && st.len() == 1
                {
                    // assert!(st.len() == 1, "{name} {st:?}", );
                    if te.tag.is_empty() {
                        te.tag = k.clone();
                    } else {
                        assert_eq!(&te.tag, k, "{ty:?}");
                    }

                    var.key = k.clone();
                    var.key = st[0].clone();

                    continue;
                }

                var.fields.push(ObjectField {
                    name: k.clone(),
                    ty: KotlinPrim::from_aty(ty),
                    required: *rq,
                });
            }

            var.name = snake_to_pascal(&var.key);

            te.variants.push(var);
        }

        te
    }
}

#[derive(Debug, Default)]
struct TaggedEnumVariant {
    name: String,
    key: String,
    fields: Vec<ObjectField>,
}

#[derive(Debug, Clone)]
struct ObjectField {
    name: String,
    ty: KotlinPrim,
    required: bool,
}

#[derive(Debug, Clone)]
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

impl KotlinPrim {
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    pub fn from_aty(ty: &ApiType) -> Self {
        if let Some(name) = &ty.name {
            return Self::Ref(name.clone());
        }

        match &ty.kind {
            ApiKind::Prim(p) => Self::from_aprim(p),
            ApiKind::Map(t) => Self::Map(Box::new(Self::from_aty(t))),
            ApiKind::Array(t) => Self::Array(Box::new(Self::from_aty(t))),
            ApiKind::Ref(name) => Self::Ref(name.clone()),
            ApiKind::Tuple(tp) => {
                let first = &tp[0];
                // if tp.iter().any(|o| o != first) {
                // }
                for o in tp {
                    assert_eq!(first, o, "{ty:?}");
                }
                Self::Array(Box::new(Self::from_aty(first)))
            }
            _ => unreachable!("{ty:?}"),
        }
    }

    pub fn gen_mfb(&self, name: &str, s: &mut String) {
        match self {
            Self::File => {
                push!(s, "mfb.addPart(", name, ")\n");
            }
            KotlinPrim::Ref(_) => {
                push!(
                    s,
                    "mfb.addPart(",
                    name,
                    ".into_json().toRequestBody(\"application/json\".toMediaType()))\n"
                );
            }
            Self::Option(opt) => {
                push!(s, "if (", name, " != null) ");
                opt.gen_mfb(name, s);
            }
            Self::Array(ar) => {
                push!(s, "for (item in ", name, ") ");
                ar.gen_mfb("item", s);
            }
            _ => unreachable!("{name} {self:#?}"),
        }
    }

    pub fn from_aprim(ap: &ApiPrim) -> Self {
        match ap {
            ApiPrim::Str => Self::Str,
            ApiPrim::Int => Self::Int,
            ApiPrim::Float => Self::Float,
            ApiPrim::Bool => Self::Bool,
            ApiPrim::File => Self::File,
            ApiPrim::Option(t) => Self::Option(Box::new(Self::from_aty(t))),
            ApiPrim::Null => unreachable!(),
        }
    }
}
