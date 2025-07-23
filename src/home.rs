use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin};

use crate::{CONTENT, Global, html, md, model::Home};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            // just load the page
            loader::glob_content(CONTENT, "index.md", yaml::<Home>),
        ])
        .add_task("home", |ctx| {
            let item = ctx.glob_one_with_file::<Content<Home>>("")?;
            let text = md::parse(&ctx, &item.data.text, &item.file.area, None)?.0;
            let html = html::home(&ctx, &text)?;
            Ok(vec![Page::text("index.html".into(), html)])
        });
});
