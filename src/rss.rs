use hauchiwa::Output;
use hauchiwa::camino::Utf8Path;
use hauchiwa::loader::Document;
use rss::{ChannelBuilder, ItemBuilder};

use crate::BASE_URL;
use crate::model::{Post, Project, Slideshow};

pub(crate) trait ToFeed: Sized {
    fn to_feed(&self) -> rss::Item;
}

fn site_url(href: &str) -> String {
    let link = href.strip_prefix("/").unwrap_or(href);
    Utf8Path::new(BASE_URL).join(link).to_string()
}

impl ToFeed for &Document<Post> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.matter.title.clone())
            .link(site_url(&self.meta.href))
            .pub_date(self.matter.date.to_rfc2822())
            .build()
    }
}

impl ToFeed for &Document<Slideshow> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.matter.title.clone())
            .link(site_url(&self.meta.href))
            .pub_date(self.matter.date.to_rfc2822())
            .build()
    }
}

impl ToFeed for &Document<Project> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.matter.title.clone())
            .link(
                self.matter
                    .link
                    .clone()
                    .unwrap_or_else(|| site_url(&self.meta.href)),
            )
            .build()
    }
}

pub fn generate_feed<'a, T>(data: &[&'a T], slug: &'static str, title: &'static str) -> Output
where
    &'a T: ToFeed + Clone,
{
    let slug = Utf8Path::new(slug);
    let data = data.iter().map(ToFeed::to_feed).collect::<Vec<_>>();

    Output::binary(
        slug.join("rss.xml"),
        ChannelBuilder::default()
            .title(title)
            .link(Utf8Path::new(BASE_URL).join(slug).to_string())
            .items(data)
            .build()
            .to_string(),
    )
}
