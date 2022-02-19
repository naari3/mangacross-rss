use comic::{Comic, Episode};
use reqwest::header::CONTENT_TYPE;
use rss::{ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, Item, ItemBuilder};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use log::info;

use crate::comic::MangaCrossComic;

mod comic;

const BOKUYABA_JSON_URL: &str = "https://mangacross.jp/api/comics/yabai.json?type=public";

async fn make_item(ep: &Episode, comic: &Comic) -> eyre::Result<Item> {
    info!("Make ep {} start", ep.sort_volume.clone());
    let mut item = ItemBuilder::default();
    let guid = GuidBuilder::default()
        .value(format!("https://mangacross.jp{}", ep.page_url))
        .permalink(true)
        .build();
    info!("Make ep {} download image start", ep.sort_volume.clone());
    let image_res = reqwest::get(ep.list_image_double_url.as_str()).await?;
    info!("Make ep {} download image done", ep.sort_volume.clone());
    let mime_type = match image_res.headers().get(CONTENT_TYPE) {
        Some(content_type) => content_type.to_str()?,
        None => "",
    };
    let length = match image_res.content_length() {
        Some(n) => n.to_string(),
        None => "".to_string(),
    };
    let enclosure = EnclosureBuilder::default()
        .url(ep.list_image_double_url.clone())
        .mime_type(mime_type)
        .length(length)
        .build();
    info!("Make ep {} done", ep.sort_volume.clone());

    Ok(item
        .title(format!("{} | {}", ep.volume.clone(), ep.title.clone()))
        .link(format!("https://mangacross.jp{}", ep.page_url))
        .guid(guid)
        .pub_date(ep.publish_start.clone())
        .author(comic.author.clone())
        .enclosure(enclosure)
        .build())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Initialized!");

    let res = reqwest::get(BOKUYABA_JSON_URL).await?;
    let body = res.text().await?;
    info!("Get bokuyaba json");

    let bokuyaba: MangaCrossComic = serde_json::from_str(body.as_str())?;
    let bokuyaba = bokuyaba.comic;

    info!("Create feed start");
    let mut channel = ChannelBuilder::default()
        .title(bokuyaba.title.clone())
        .link(format!(
            "https://mangacross.jp/comics/{}/",
            bokuyaba.dir_name.clone()
        ))
        .description(bokuyaba.caption_for_search.clone())
        .image(
            ImageBuilder::default()
                .url(bokuyaba.image_url.clone())
                .link(bokuyaba.image_url.clone())
                .title(format!("{} {}", &bokuyaba.title, &bokuyaba.author))
                .build(),
        )
        .pub_date(bokuyaba.latest_episode_publish_start.clone())
        .last_build_date(bokuyaba.latest_episode_publish_start.clone())
        .build();

    let items: Vec<Item> =
        futures::future::try_join_all(bokuyaba.episodes.iter().map(|ep| make_item(ep, &bokuyaba)))
            .await?;
    channel.set_items(items);

    let feed = channel.to_string();
    info!("Create feed done");

    info!("Create feed.xml");
    let mut file = File::create("feed.xml").await?;
    info!("Write feed.xml");
    file.write_all(feed.as_bytes()).await?;
    info!("Done");

    Ok(())
}
