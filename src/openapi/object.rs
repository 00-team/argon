#![allow(dead_code)]

use indexmap::IndexMap;
use serde::Deserialize;

use super::{
    common::*,
    format::{KnownFormat, SchemaFormat},
};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Number {
    /// Signed integer e.g. `1` or `-2`
    Int(isize),
    /// Unsigned integer value e.g. `0`. Unsigned integer cannot be below zero.
    UInt(usize),
    /// Floating point number e.g. `1.34`
    Float(f64),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum AdditionalProperties<T> {
    /// Use when value type of the map is a known [`Schema`] or [`Ref`] to the [`Schema`].
    RefOr(RefOr<T>),
    /// Use _`AdditionalProperties::FreeForm(true)`_ when any value is allowed in the map.
    FreeForm(bool),
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub title: Option<String>,
    pub format: Option<SchemaFormat>,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: IndexMap<String, RefOr<OaSchema>>,

    /// Additional [`Schema`] for non specified fields (Useful for typed maps).
    pub additional_properties: Option<Box<AdditionalProperties<OaSchema>>>,

    /// Additional [`Schema`] to describe property names of an object such as a map. See more
    /// details <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-propertynames>
    pub property_names: Option<Box<OaSchema>>,
    pub deprecated: Option<bool>,

    // pub examples: Vec<serde_json::Value>,
    /// Write only property will be only sent in _write_ requests like _POST, PUT_.
    // pub write_only: Option<bool>,

    /// Read only property will be only sent in _read_ requests like _GET_.
    // pub read_only: Option<bool>,

    /// Additional [`Xml`] formatting of the [`Object`].
    // pub xml: Option<Xml>,

    /// Must be a number strictly greater than `0`. Numeric value is considered valid if value
    /// divided by the _`multiple_of`_ value results an integer.
    pub multiple_of: Option<Number>,

    /// Specify inclusive upper limit for the [`Object`]'s value. Number is considered valid if
    /// it is equal or less than the _`maximum`_.
    pub maximum: Option<Number>,

    /// Specify inclusive lower limit for the [`Object`]'s value. Number value is considered
    /// valid if it is equal or greater than the _`minimum`_.
    pub minimum: Option<Number>,

    /// Specify exclusive upper limit for the [`Object`]'s value. Number value is considered
    /// valid if it is strictly less than _`exclusive_maximum`_.
    pub exclusive_maximum: Option<Number>,

    /// Specify exclusive lower limit for the [`Object`]'s value. Number value is considered
    /// valid if it is strictly above the _`exclusive_minimum`_.
    pub exclusive_minimum: Option<Number>,

    /// Specify maximum length for `string` values. _`max_length`_ cannot be a negative integer
    /// value. Value is considered valid if content length is equal or less than the _`max_length`_.
    pub max_length: Option<usize>,

    /// Specify minimum length for `string` values. _`min_length`_ cannot be a negative integer
    /// value. Setting this to _`0`_ has the same effect as omitting this field. Value is
    /// considered valid if content length is equal or more than the _`min_length`_.
    pub min_length: Option<usize>,

    /// Define a valid `ECMA-262` dialect regular expression. The `string` content is
    /// considered valid if the _`pattern`_ matches the value successfully.
    pub pattern: Option<String>,

    /// Specify inclusive maximum amount of properties an [`Object`] can hold.
    pub max_properties: Option<usize>,

    /// Specify inclusive minimum amount of properties an [`Object`] can hold. Setting this to
    /// `0` will have same effect as omitting the attribute.
    pub min_properties: Option<usize>,

    /// Optional extensions `x-something`.
    // #[serde(flatten)]
    // pub extensions: Option<Extensions>,

    /// The `content_encoding` keyword specifies the encoding used to store the contents, as specified in
    /// [RFC 2054, part 6.1](https://tools.ietf.org/html/rfc2045) and [RFC 4648](RFC 2054, part 6.1).
    ///
    /// Typically this is either unset for _`string`_ content types which then uses the content
    /// encoding of the underlying JSON document. If the content is in _`binary`_ format such as an image or an audio
    /// set it to `base64` to encode it as _`Base64`_.
    ///
    /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentencoding>
    #[serde(default)]
    pub content_encoding: String,

    /// The _`content_media_type`_ keyword specifies the MIME type of the contents of a string,
    /// as described in [RFC 2046](https://tools.ietf.org/html/rfc2046).
    ///
    /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentmediatype>
    #[serde(default)]
    pub content_media_type: String,
}

impl Def for Object {
    fn def_ts<'a, F: GetRef<'a>>(&self, get_ref: &F) -> String {
        let ty = match &self.schema_type {
            SchemaType::AnyValue => return "any".to_string(),
            SchemaType::Array(v) => {
                if v[0] == Type::String && v[1] == Type::Null {
                    return "string | null".to_string();
                }
                todo!("type array: {v:?}");
            }
            SchemaType::Type(t) => t,
        };

        // if let Some(ev) = &self.enum_values {
        //     println!("{:?} / ev: {ev:#?}", self.schema_type);
        // }

        match ty {
            Type::String => {
                if let Some(ev) = &self.enum_values {
                    let mut uni = Vec::<String>::with_capacity(ev.len());
                    for v in ev {
                        uni.push(format!(r##""{}""##, v.as_str().unwrap()));
                    }
                    return uni.join("|");
                }

                if let Some(SchemaFormat::KnownFormat(KnownFormat::Binary)) =
                    &self.format
                {
                    return "File".to_string();
                }

                "string".to_string()
            }
            Type::Object => {
                let mut o = String::from("{\n");
                for (k, v) in self.properties.iter() {
                    let xv = match v {
                        RefOr::T(t) => t.def_ts(get_ref),
                        RefOr::Ref(r) => match get_ref(r) {
                            Some((i, _)) => i,
                            None => "any".to_string(),
                        },
                    };
                    o.push_str("    ");
                    o.push_str(k);
                    o.push_str(": ");
                    o.push_str(&xv);
                    // if xv == "Gene" {
                    //     o.push_str(" | null");
                    // }
                    o.push_str(",\n");
                }
                o.push_str("}\n");
                o
            }
            Type::Integer | Type::Number => {
                // println!("int: {self:#?}");
                "number".to_string()
            }
            Type::Boolean => "boolean".to_string(),
            Type::Null => "null".to_string(),
            Type::Array => {
                "[]".to_string()
                // println!("{self:#?}");
                // todo!("hi")
            }
        }
    }

    fn is_user_defined(&self) -> bool {
        if let Some(desc) = &self.description {
            if desc.contains("#user_defined") {
                return true;
            }
        }

        if let Some(tit) = &self.title {
            if tit.contains("#user_defined") {
                return true;
            }
        }

        false
    }
}
