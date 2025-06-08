use super::common::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ArrayItems {
    R(Box<RefOr<OaSchema>>),
    #[serde(with = "array_items_false")]
    False,
}

mod array_items_false {
    use serde::de::Visitor;

    pub fn serialize<S: serde::Serializer>(
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(false)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<(), D::Error> {
        struct ItemsFalseVisitor;

        impl<'de> Visitor<'de> for ItemsFalseVisitor {
            type Value = ();
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !v {
                    Ok(())
                } else {
                    Err(serde::de::Error::custom(format!(
                        "invalid boolean value: {v}, expected false"
                    )))
                }
            }

            fn expecting(
                &self, formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("expected boolean false")
            }
        }

        deserializer.deserialize_bool(ItemsFalseVisitor)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub title: Option<String>,
    pub items: ArrayItems,
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
        // if !matches!(self.schema_type, SchemaType::Type(Type::Array)) {
        //     panic!("wtf array: {self:#?}");
        // }
        let mut or_null = String::with_capacity(32);
        match &self.schema_type {
            SchemaType::Type(Type::Array) => {}
            SchemaType::Array(vt) => {
                if vt[0] == Type::Array && vt[1] == Type::Null {
                    or_null.push_str(" | null");
                } else {
                    todo!("schema array: {vt:?}");
                }
            }
            _ => panic!("unknown array type: {self:?}"),
        }

        let ArrayItems::R(r) = &self.items else {
            let items = self
                .prefix_items
                .iter()
                .map(|s| s.def_ts(get_ref))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("(([{items}]){or_null})");
        };

        let xv = match &(**r) {
            RefOr::T(t) => t.def_ts(get_ref),
            RefOr::Ref(r) => match get_ref(r) {
                Some((i, _)) => i,
                None => format!("any{or_null}"),
            },
        };

        let xv = if xv == "Gene" { "(Gene | null)".to_string() } else { xv };

        if let Some(min) = self.min_items {
            let fixed = self.max_items.map(|v| v == min).unwrap_or(false);
            let t = (0..min).map(|_| xv.as_str()).collect::<Vec<_>>().join(",");
            if fixed {
                return format!("(([{t}]){or_null})");
            }
            return format!("(([{t}, ...({xv}[])]){or_null})");
        }

        format!("(({xv}[]){or_null})")
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
