use serde::Deserialize;

use super::{
    array::Array,
    object::Object,
    of::{AllOf, OneOf},
};

pub trait GetRef<'a>:
    Fn(&Ref) -> Option<(String, &'a RefOr<OaSchema>)>
{
}
impl<'a, T: Fn(&Ref) -> Option<(String, &'a RefOr<OaSchema>)>> GetRef<'a>
    for T
{
}

pub trait Def {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String;
    fn is_user_defined(&self) -> bool;
}

#[derive(Debug, Deserialize)]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub loc: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum OaSchema {
    Array(Array),
    Object(Object),
    OneOf(OneOf),
    AllOf(AllOf),
    AnyOf,
}

impl Def for OaSchema {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        let x = match self {
            Self::Object(o) => o.def_ts(get_ref),
            Self::AllOf(a) => a.def_ts(get_ref),
            Self::OneOf(a) => a.def_ts(get_ref),
            Self::Array(a) => a.def_ts(get_ref),
            Self::AnyOf => todo!("any of ??"),
        };
        format!("({x})")
    }
    fn is_user_defined(&self) -> bool {
        match self {
            Self::Object(o) => o.is_user_defined(),
            Self::AllOf(o) => o.is_user_defined(),
            Self::OneOf(o) => o.is_user_defined(),
            Self::Array(o) => o.is_user_defined(),
            Self::AnyOf => false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SchemaType {
    Type(Type),
    Array(Vec<Type>),
    AnyValue,
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::Type(Type::default())
    }
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[default]
    Object,
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Null,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(Ref),
    T(T),
}

impl<T: Def> Def for RefOr<T> {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        let x = match self {
            Self::T(t) => t.def_ts(get_ref),
            Self::Ref(r) => {
                let i = r.loc.split('/').last().unwrap();
                i.to_string()
            }
        };
        format!("({x})")
    }
    fn is_user_defined(&self) -> bool {
        match self {
            Self::T(t) => t.is_user_defined(),
            Self::Ref(_) => false,
        }
    }
}
