#![allow(dead_code)]

mod object;
pub use object::*;

pub trait Def {
    fn def_ts<'a, F: GetRef<'a>>(&self, _get_ref: &F) -> String;
}

use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, io::Write};

#[derive(Debug, Deserialize)]
pub struct OpenApi {
    pub paths: HashMap<String, OaPath>,
    pub components: OaComponents,
}

#[derive(Debug, Deserialize)]
pub struct OaComponents {
    pub schemas: HashMap<String, RefOr<OaSchema>>,
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
#[serde(rename_all = "lowercase", untagged)]
pub enum SchemaFormat {
    /// Use to define additional detail about the value.
    KnownFormat(KnownFormat),
    /// Can be used to provide additional detail about the value when [`SchemaFormat::KnownFormat`]
    /// is not suitable.
    Custom(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnownFormat {
    /// 8 bit integer.
    Int8,
    /// 16 bit integer.
    Int16,
    /// 32 bit integer.
    Int32,
    /// 64 bit integer.
    Int64,
    /// 8 bit unsigned integer.
    UInt8,
    /// 16 bit unsigned integer.
    UInt16,
    /// 32 bit unsigned integer.
    UInt32,
    /// 64 bit unsigned integer.
    UInt64,
    /// floating point number.
    Float,
    /// double (floating point) number.
    Double,
    /// base64 encoded chars.
    Byte,
    /// binary data (octet).
    Binary,
    /// ISO-8601 full time format [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Time,
    /// ISO-8601 full date [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Date,
    /// ISO-8601 full date time [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    DateTime,
    /// duration format from [RFC3339 Appendix-A](https://datatracker.ietf.org/doc/html/rfc3339#appendix-A).
    Duration,
    /// Hint to UI to obscure input.
    Password,
    /// Used with [`String`] values to indicate value is in UUID format.
    ///
    /// **uuid** feature need to be enabled.
    Uuid,
    /// Used with [`String`] values to indicate value is in ULID format.
    Ulid,
    /// Used with [`String`] values to indicate value is in Url format according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Uri,
    /// A string instance is valid against this attribute if it is a valid URI Reference
    /// (either a URI or a relative-reference) according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    UriReference,
    /// A string instance is valid against this attribute if it is a
    /// valid IRI, according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    Iri,
    /// A string instance is valid against this attribute if it is a valid IRI Reference
    /// (either an IRI or a relative-reference)
    /// according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    IriReference,
    /// As defined in "Mailbox" rule [RFC5321](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.2).
    Email,
    /// As defined by extended "Mailbox" rule [RFC6531](https://datatracker.ietf.org/doc/html/rfc6531#section-3.3).
    IdnEmail,
    /// As defined by [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1), including host names
    /// produced using the Punycode algorithm
    /// specified in [RFC5891](https://datatracker.ietf.org/doc/html/rfc5891#section-4.4).
    Hostname,
    /// As defined by either [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1) as for hostname,
    /// or an internationalized hostname as defined by [RFC5890](https://datatracker.ietf.org/doc/html/rfc5890#section-2.3.2.3).
    IdnHostname,
    /// An IPv4 address according to [RFC2673](https://datatracker.ietf.org/doc/html/rfc2673#section-3.2).
    Ipv4,
    /// An IPv6 address according to [RFC4291](https://datatracker.ietf.org/doc/html/rfc4291#section-2.2).
    Ipv6,
    /// A string instance is a valid URI Template if it is according to
    /// [RFC6570](https://datatracker.ietf.org/doc/html/rfc6570).
    ///
    /// _**Note!**_ There are no separate IRL template.
    UriTemplate,
    /// A valid JSON string representation of a JSON Pointer according to [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901#section-5).
    JsonPointer,
    /// A valid relative JSON Pointer according to [draft-handrews-relative-json-pointer-01](https://datatracker.ietf.org/doc/html/draft-handrews-relative-json-pointer-01).
    RelativeJsonPointer,
    /// Regular expression, which SHOULD be valid according to the
    /// [ECMA-262](https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#ref-ecma262).
    Regex,
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
}

#[derive(Debug, Deserialize)]
pub struct OaPath {}

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
}

#[derive(Debug, Deserialize)]
pub struct OneOf {
    #[serde(rename = "oneOf")]
    pub items: Vec<RefOr<OaSchema>>,
    #[serde(rename = "type", default)]
    pub schema_type: SchemaType,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Def for OneOf {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        let mut uni = Vec::<String>::with_capacity(self.items.len());
        for i in self.items.iter() {
            uni.push(i.def_ts(get_ref));
        }

        uni.join("|")
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

#[derive(Debug, Deserialize, Default)]
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

// #[derive(Debug, Deserialize)]
// pub struct AnyOf {
//     #[serde(rename = "anyOf")]
//     pub items: Vec<RefOr<OaSchema>>,
//     #[serde(rename = "type", default)]
//     pub schema_type: SchemaType,
//     pub description: Option<String>,
// }

#[derive(Debug, Deserialize)]
pub struct AllOf {
    #[serde(rename = "allOf")]
    pub items: Vec<RefOr<OaSchema>>,
    #[serde(rename = "type", default)]
    pub schema_type: SchemaType,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Def for AllOf {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        let mut and = Vec::<String>::with_capacity(self.items.len());
        for i in self.items.iter() {
            and.push(i.def_ts(get_ref));
        }

        and.join("&")
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub title: Option<String>,
    pub items: Box<RefOr<OaSchema>>,
    #[serde(default)]
    pub prefix_items: Vec<OaSchema>,
    pub description: Option<String>,
    pub deprecated: Option<bool>,
    pub max_items: Option<usize>,
    pub min_items: Option<usize>,

    #[serde(default)]
    pub unique_items: bool,

    #[serde(default)]
    pub content_encoding: String,

    #[serde(default)]
    pub content_media_type: String,
}

impl Def for Array {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        assert!(matches!(self.schema_type, SchemaType::Type(Type::Array)));

        let xv = match &(*self.items) {
            RefOr::T(t) => t.def_ts(get_ref),
            RefOr::Ref(r) => if get_ref(r).is_some() {
                r.loc.split('/').last().unwrap()
            } else {
                "any"
            }
            .to_string(),
        };

        format!("({xv}[])")
    }
}

pub trait GetRef<'a>: Fn(&Ref) -> Option<&'a RefOr<OaSchema>> {}
impl<'a, T: Fn(&Ref) -> Option<&'a RefOr<OaSchema>>> GetRef<'a> for T {}

pub fn decode() -> std::io::Result<()> {
    let mut ts = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.ts")?;

    let oa: OpenApi = serde_json::from_str(&read_to_string("openapi.json")?)?;

    std::fs::write("out.txt", format!("{:#?}", oa.components.schemas))?;

    let get_ref = |loc: &Ref| {
        let i = loc.loc.split('/').last().unwrap();
        oa.components.schemas.get(i)
    };

    for (ident, s) in oa.components.schemas.iter() {
        let RefOr::T(s) = s else { continue };

        let def = s.def_ts(&get_ref);
        ts.write_all(format!("export type {ident} = {def};\n").as_bytes())?;
    }

    Ok(())
}
