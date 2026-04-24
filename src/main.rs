use std::fs::read_to_string;

use openapi::OpenApi;

mod kotlin;
mod models;
mod openapi;

fn main() -> std::io::Result<()> {
    let oas = read_to_string("argon-data/openapi.json")?;
    let oa: OpenApi = serde_json::from_str(&oas)?;

    // openapi::generate(&oa)?;
    let asp = models::ApiSchema::from_openapi(&oa);
    asp.generate()?;

    let kapi = kotlin::KotlinApi::new(&asp);
    kapi.generate()?;

    Ok(())
}
