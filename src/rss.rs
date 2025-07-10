use camino::Utf8Path;
use hauchiwa::loader::Content;
use hauchiwa::{Page, TaskResult, WithFile};
use rss::{ChannelBuilder, ItemBuilder};

use crate::model::{Post, Project};
use crate::{BASE_URL, Context, Slideshow};

pub(crate) trait ToFeed: Sized {
    fn to_feed(&self) -> rss::Item;
}

impl ToFeed for WithFile<'_, Content<Post>> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.data.meta.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&self.file.slug).to_string())
            .build()
    }
}

impl ToFeed for WithFile<'_, Content<Slideshow>> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.data.meta.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&self.file.slug).to_string())
            .build()
    }
}

impl ToFeed for WithFile<'_, Content<Project>> {
    fn to_feed(&self) -> rss::Item {
        ItemBuilder::default()
            .title(self.data.meta.title.clone())
            .link(Utf8Path::new(BASE_URL).join(&self.file.slug).to_string())
            .build()
    }
}

pub(crate) fn generate_feed<T>(
    ctx: Context,
    slug: &'static str,
    title: &'static str,
) -> TaskResult<Page>
where
    T: 'static,
    for<'a> WithFile<'a, T>: ToFeed,
{
    let slug = Utf8Path::new(slug);
    let glob = slug.join("**/*");

    let items = ctx
        .glob_with_files::<T>(glob.as_str())?
        .iter()
        .map(ToFeed::to_feed)
        .collect::<Vec<_>>();

    Ok(Page::text(
        slug.join("rss.xml"),
        ChannelBuilder::default()
            .title(title)
            .link(Utf8Path::new(BASE_URL).join(slug).to_string())
            .items(items)
            .build()
            .to_string(),
    ))
}
