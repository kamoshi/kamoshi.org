use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin};

use crate::model::Slideshow;
use crate::{CONTENT, Global, LinkDate, render_page_slideshow};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            loader::glob_content(CONTENT, "slides/**/*.md", yaml::<Slideshow>),
            loader::glob_content(CONTENT, "slides/**/*.lhs", yaml::<Slideshow>),
        ])
        .add_task("slides", |sack| {
            let pages = sack
                .glob_with_file::<Content<Slideshow>>("slides/**/*")?
                .into_iter()
                .map(|query| render_page_slideshow(&sack, query))
                .collect::<Result<_, _>>()?;
            Ok(pages)
        })
        .add_task("slides_list", |sack| {
            Ok(vec![Page::text(
                "slides/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.glob_with_file::<Content<Slideshow>>("slides/**/*")?
                        .into_iter()
                        .map(LinkDate::from)
                        .collect(),
                    "Slideshows".into(),
                    "/slides/rss.xml",
                ),
            )])
        })
        .add_task("slides_feed", |sack| {
            let feed = crate::rss::generate_feed::<Content<Slideshow>>(
                sack,
                "slides",
                "Kamoshi.org Slides",
            )?;
            Ok(vec![feed])
        });
});
