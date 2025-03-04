mod openapi;

fn main() -> std::io::Result<()> {
    openapi::generate()?;

    Ok(())
}
