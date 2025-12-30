use hauchiwa::Output;
use hauchiwa::camino::Utf8Path;
use hauchiwa::loader::Document;
use hauchiwa::page::normalize_prefixed;
use rss::{ChannelBuilder, ItemBuilder};

use crate::BASE_URL;
use crate::model::{Post, Project, Slideshow};

pub(crate) trait ToFeed: Sized {
    fn to_feed(&self) -> rss::Item;
}

impl ToFeed for &Document<Post> {
    fn to_feed(&self) -> rss::Item {
        let path = normalize_prefixed("content/", &self.path);
        ItemBuilder::default()
            .title(self.metadata.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&path).to_string())
            .build()
    }
}

impl ToFeed for &Document<Slideshow> {
    fn to_feed(&self) -> rss::Item {
        let path = normalize_prefixed("content/", &self.path);
        ItemBuilder::default()
            .title(self.metadata.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&path).to_string())
            .build()
    }
}

impl ToFeed for &Document<Project> {
    fn to_feed(&self) -> rss::Item {
        let path = normalize_prefixed("content/", &self.path);
        ItemBuilder::default()
            .title(self.metadata.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&path).to_string())
            .build()
    }
}

pub fn generate_feed<'a, T>(data: &[&'a T], slug: &'static str, title: &'static str) -> Output
where
    &'a T: ToFeed + Clone,
{
    let slug = Utf8Path::new(slug);
    let data = data.iter().map(ToFeed::to_feed).collect::<Vec<_>>();

    Output::file(
        slug.join("rss.xml"),
        ChannelBuilder::default()
            .title(title)
            .link(Utf8Path::new(BASE_URL).join(slug).to_string())
            .items(data)
            .build()
            .to_string(),
    )
}
