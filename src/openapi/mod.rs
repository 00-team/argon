#![allow(dead_code)]

mod array;
mod common;
mod format;
mod object;
mod of;
mod path;

use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, io::Write};

use self::common::{Def, Ref, RefOr};

#[derive(Debug, Deserialize)]
pub struct OpenApi {
    pub paths: HashMap<String, path::OaPath>,
    pub components: OaComponents,
}

#[derive(Debug, Deserialize)]
pub struct OaComponents {
    pub schemas: HashMap<String, common::RefOr<common::OaSchema>>,
}

pub fn generate() -> std::io::Result<()> {
    let mut ts = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.ts")?;

    let oa: OpenApi = serde_json::from_str(&read_to_string("openapi.json")?)?;

    std::fs::write("out.txt", format!("{:#?}", oa.paths))?;

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
