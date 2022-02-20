use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use log::info;

use crate::manga_cross::MangaCrossComic;

mod manga_cross;

const BOKUYABA_JSON_URL: &str = "https://mangacross.jp/api/comics/yabai.json?type=public";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Initialized!");

    let res = reqwest::get(BOKUYABA_JSON_URL).await?;
    let body = res.text().await?;
    info!("Get bokuyaba json");

    let bokuyaba: MangaCrossComic = serde_json::from_str(body.as_str())?;

    info!("Create feed start");
    let channel = bokuyaba.to_channel().await?;
    let feed = channel.to_string();
    info!("Create feed done");

    info!("Write to feed.xml");
    let mut file = File::create("feed.xml").await?;
    file.write_all(feed.as_bytes()).await?;
    info!("Done");

    Ok(())
}
