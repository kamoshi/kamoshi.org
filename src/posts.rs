use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin};

use crate::model::Post;
use crate::{CONTENT, Global, LinkDate, render_page_post};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "posts/**/*.md", yaml::<Post>),
        ])
        .add_task("posts", |ctx| {
            let pages = ctx
                .glob_with_file::<Content<Post>>("posts/**/*")?
                .into_iter()
                .filter(|item| !item.data.meta.draft)
                .map(|query| render_page_post(&ctx, query))
                .collect::<Result<_, _>>()?;
            Ok(pages)
        })
        .add_task("posts_list", |ctx| {
            Ok(vec![Page::text(
                "posts/index.html".into(),
                crate::html::to_list(
                    &ctx,
                    ctx.glob_with_file::<Content<Post>>("posts/**/*")?
                        .iter()
                        .filter(|item| !item.data.meta.draft)
                        .map(LinkDate::from)
                        .collect(),
                    "Posts".into(),
                    "/posts/rss.xml",
                ),
            )])
        })
        .add_task("posts_feed", |sack| {
            let feed =
                crate::rss::generate_feed::<Content<Post>>(sack, "posts", "Kamoshi.org Posts")?;
            Ok(vec![feed])
        });
});
