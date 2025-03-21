use super::common::*;
use serde::Deserialize;

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
            RefOr::Ref(r) => match get_ref(r) {
                Some((i, _)) => i,
                None => "any".to_string(),
            },
        };

        format!("({xv}[])")
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
