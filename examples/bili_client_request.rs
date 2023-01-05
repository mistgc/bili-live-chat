#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://www.bilibili.com").await?;
    println!("{:#?}", resp);

    Ok(())
}
