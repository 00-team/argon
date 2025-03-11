#![allow(dead_code)]

pub mod array;
pub mod common;
pub mod format;
pub mod object;
pub mod of;
pub mod path;

use indoc::formatdoc;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use self::common::{Def, Ref, RefOr};

#[derive(Debug, Deserialize)]
pub struct OpenApi {
    pub paths: HashMap<String, path::PathItem>,
    pub components: OaComponents,
}

#[derive(Debug, Deserialize)]
pub struct OaComponents {
    pub schemas: HashMap<String, common::RefOr<common::OaSchema>>,
}

pub fn generate(oa: &OpenApi) -> std::io::Result<()> {
    let mut ts = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("data/out.ts")?;

    let get_ref = |loc: &Ref| {
        let i = loc.loc.split('/').last().unwrap();
        match oa.components.schemas.get(i) {
            Some(v) => Some((i.to_string(), v)),
            None => None,
        }
    };

    ts.write_all(
        formatdoc! {"
            import * as ud from './user_defined';
        "}
        .as_bytes(),
    )?;

    for (ident, s) in oa.components.schemas.iter() {
        let RefOr::T(s) = s else { continue };

        let def = s.def_ts(&get_ref);
        if s.is_user_defined() {
            continue;
        }
        ts.write_all(format!("export type {ident} = {def};\n").as_bytes())?;
    }

    let mut names = HashSet::<String>::with_capacity(oa.paths.len());
    let has_name = |name: &str| name.contains(name);

    for (url, p) in oa.paths.iter() {
        macro_rules! dop {
            ($name:ident) => {
                if let Some(op) = &p.$name {
                    let (def, name) =
                        op.def_ts(url, stringify!($name), &get_ref, &has_name);
                    names.insert(name);
                    ts.write_all(def.as_bytes())?;
                }
            };
        }
        dop!(get);
        dop!(put);
        dop!(post);
        dop!(delete);
        dop!(patch);
    }

    Ok(())
}
