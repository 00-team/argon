use super::common::*;
use serde::Deserialize;

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

    fn is_user_defined(&self) -> bool {
        if let Some(s) = &self.description {
            if s.contains("#user_defined") {
                return true;
            }
        }

        if let Some(s) = &self.title {
            if s.contains("#user_defined") {
                return true;
            }
        }

        false
    }
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

    fn is_user_defined(&self) -> bool {
        if let Some(s) = &self.description {
            if s.contains("#user_defined") {
                return true;
            }
        }

        if let Some(s) = &self.title {
            if s.contains("#user_defined") {
                return true;
            }
        }

        false
    }
}
