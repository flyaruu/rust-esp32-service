
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let _a = 1;
    println!("Hello, world!");
    Ok(())
}
