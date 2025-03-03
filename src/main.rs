mod openapi;

fn main() -> std::io::Result<()> {
    openapi::decode()?;

    Ok(())
}
