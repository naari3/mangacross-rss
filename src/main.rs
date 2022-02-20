use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use log::info;
use tokio::time::Instant;

use crate::mangacross::MangaCrossComic;

mod mangacross;

const TARGETS: [&str; 2] = ["yabai", "shiomai"];

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let start = Instant::now();
    info!("Initialized!");

    for target in TARGETS {
        let channel_start = Instant::now();
        info!("Start {target}");
        let url = format!("https://mangacross.jp/api/comics/{target}.json?type=public",);
        let res = reqwest::get(url).await?;
        let body = res.text().await?;
        info!("Get {target} json");

        let comic: MangaCrossComic = serde_json::from_str(body.as_str())?;

        info!("Create feed start");
        let channel = comic.to_channel().await?;
        let feed = channel.to_string();
        info!("Create feed done");

        info!("Create public/{} dir", target);
        fs::create_dir_all(format!("public/{target}")).await?;

        info!("Write to feed.xml");
        let mut file = File::create(format!("public/{target}/feed.xml")).await?;
        file.write_all(feed.as_bytes()).await?;
        let channel_end = channel_start.elapsed();
        info!(
            "Done {target}. {}.{:03} secs elapsed.",
            channel_end.as_secs(),
            channel_end.subsec_nanos() / 1_000_000
        );
    }

    info!("Copy index.html");
    fs::copy("./index.html", "public/index.html").await?;

    let end = start.elapsed();
    info!(
        "Done. {}.{:03} secs elapsed.",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );

    Ok(())
}
