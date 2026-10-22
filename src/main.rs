use std::fs::read_to_string;

use openapi::OpenApi;

mod kotlin;
mod models;
mod openapi;
mod utils;

fn main() -> std::io::Result<()> {
    let oas = read_to_string("argon-data/openapi.json")?;
    let oa: OpenApi = serde_json::from_str(&oas)?;
    println!("paths: {}", oa.paths.len());

    let no_kotlin =
        std::env::args().into_iter().any(|a| a.contains("no-kotlin"));

    // openapi::generate(&oa)?;
    let asp = models::ApiSchema::from_openapi(&oa);
    asp.generate()?;

    if !no_kotlin {
        let kapi = kotlin::KotlinApi::new(&asp);
        kapi.generate()?;
        kapi.gen_routes()?;
    }

    Ok(())
}
