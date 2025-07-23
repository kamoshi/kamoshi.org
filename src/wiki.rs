use hauchiwa::Plugin;
use hauchiwa::loader::{self, Content, yaml};

use crate::model::Wiki;
use crate::{CONTENT, Global, render_page_wiki};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "wiki/**/*.md", yaml::<Wiki>),
        ])
        .add_task("wiki", |ctx| {
            let mut pages = vec![];

            for item in ctx.glob_with_file::<Content<Wiki>>("**/*")? {
                pages.push(render_page_wiki(&ctx, item)?);
            }

            Ok(pages)
        });
});
