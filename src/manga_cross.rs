use log::info;
use reqwest::header::CONTENT_TYPE;
use rss::{
    Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, Item, ItemBuilder,
};
use serde::{Deserialize, Serialize};

const MANGACROSS_HOST: &str = "https://mangacross.jp";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MangaCrossComic {
    pub comic: Comic,
}

impl MangaCrossComic {
    pub async fn to_channel(&self) -> eyre::Result<Channel> {
        let comic = &self.comic;

        info!("to_channel {} start", comic.title);
        let mut channel = ChannelBuilder::default()
            .title(comic.title.clone())
            .link(format!(
                "{}/comics/{}/",
                MANGACROSS_HOST,
                comic.dir_name.clone()
            ))
            .description(comic.caption_for_search.clone())
            .image(
                ImageBuilder::default()
                    .url(comic.image_url.clone())
                    .link(comic.image_url.clone())
                    .title(format!("{} {}", &comic.title, &comic.author))
                    .build(),
            )
            .pub_date(comic.latest_episode_publish_start.clone())
            .last_build_date(comic.latest_episode_publish_start.clone())
            .build();

        let items: Vec<Item> = futures::future::try_join_all(
            comic
                .episodes
                .iter()
                .filter(|ep| ep.status == "public")
                .map(|ep| ep.to_item(&comic)),
        )
        .await?;
        channel.set_items(items);

        info!("to_channel {} done", comic.title);
        Ok(channel)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comic {
    pub dir_name: String,
    pub title: String,
    pub title_kana: String,
    pub author: String,
    pub author_kana: String,
    pub comic_category: ComicCategory,
    pub comic_tags: Vec<ComicTag>,
    pub image_url: String,
    pub image_double_url: String,
    pub list_image_url: String,
    pub list_image_double_url: String,
    pub caption: String,
    pub caption_for_search: String,
    pub latest_episode_publish_start: String,
    pub restricted: bool,
    pub series: bool,
    pub seo_word_common: String,
    pub seo_word_comic: String,
    pub seo_word_episode: String,
    pub seo_outline: String,
    pub ad_lating: usize,
    pub outline: String,
    pub comic_url: String,
    pub large_image_url: String,
    pub image_sp_url: String,
    pub logo_url: String,
    pub background_url: String,
    pub ogp_url: String,
    pub icon_url: String,
    pub tw_hashtag: String,
    pub tw_screen_name: String,
    pub next_publish_at: String,
    pub next_date_customize_text: Option<String>,
    pub promotion: Promotion,
    pub is_unlimited_comic: bool,
    // pub unlimited_event_singles: Vec<?>,
    pub episodes: Vec<Episode>,
    pub books: Vec<Book>,
    pub related_comics: Vec<RelatedComic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComicCategory {
    pub name: String,
    pub display_name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComicTag {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Promotion {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Episode {
    pub id: usize,
    pub volume: String,
    pub sort_volume: usize,
    pub page_count: usize,
    pub title: String,
    pub publish_start: String,
    pub publish_end: Option<String>,
    pub member_publish_start: String,
    pub member_publish_end: Option<String>,
    pub status: String,
    pub page_url: String,
    pub ogp_url: String,
    pub list_image_url: String,
    pub list_image_double_url: String,
    pub episode_next_date: Option<String>,
    pub next_date_customize_text: Option<String>,
    pub is_unlimited_comic: bool,
}

impl Episode {
    pub async fn to_item(&self, comic: &Comic) -> eyre::Result<Item> {
        info!("to_item {} start", self.sort_volume);
        let mut item = ItemBuilder::default();
        let guid = GuidBuilder::default()
            .value(format!("{}{}", MANGACROSS_HOST, self.page_url))
            .permalink(true)
            .build();
        info!("to_item {} download image start", self.sort_volume);
        let image_res = reqwest::get(self.list_image_double_url.as_str()).await?;
        info!("to_item {} download image done", self.sort_volume);
        let mime_type = match image_res.headers().get(CONTENT_TYPE) {
            Some(content_type) => content_type.to_str()?,
            None => "",
        };
        let length = match image_res.content_length() {
            Some(n) => n.to_string(),
            None => "".to_string(),
        };
        let enclosure = EnclosureBuilder::default()
            .url(self.list_image_double_url.clone())
            .mime_type(mime_type)
            .length(length)
            .build();
        info!("to_item {} done", self.sort_volume);

        Ok(item
            .title(format!("{} | {}", self.volume, self.title))
            .link(format!("{}{}", MANGACROSS_HOST, self.page_url))
            .guid(guid)
            .pub_date(self.publish_start.clone())
            .author(comic.author.clone())
            .enclosure(enclosure)
            .build())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Book {
    pub id: usize,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub cover_url: String,
    pub release_date: String,
    pub purchase_url: PurchaseUrl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurchaseUrl {
    pub amazon: String,
    pub rakuten: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelatedComic {
    pub dir_name: String,
    pub title: String,
    pub title_kana: String,
    pub author: String,
    pub author_kana: String,
    pub comic_category: ComicCategory,
    pub comic_tags: Vec<ComicTag>,
    pub image_url: String,
    pub image_double_url: String,
    pub list_image_url: String,
    pub list_image_double_url: String,
    pub caption: String,
    pub caption_for_search: String,
    pub latest_episode_publish_start: Option<String>,
    pub restricted: bool,
}
