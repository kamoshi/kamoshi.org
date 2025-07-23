use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin};

use crate::model::Project;
use crate::{CONTENT, Global};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "projects/**/*.md", yaml::<Project>),
        ])
        .add_task("projects", |ctx| {
            let mut pages = vec![];

            let data = ctx.glob_with_file::<Content<Project>>("projects/**/*")?;
            let list = crate::html::project::render_list(&ctx, data)?;
            pages.push(Page::text("projects/index.html".into(), list));

            let text = ctx.get::<String>("hauchiwa")?;
            let (text, outline, _) = crate::md::parse(&ctx, text, "".into(), None)?;
            let html = crate::html::project::render_page(&ctx, &text, outline)?;
            pages.push(Page::text("projects/hauchiwa/index.html".into(), html));

            Ok(pages)
        })
        // .add_task(|sack| {
        //     let query = sack.get_content("projects/flox")?;
        //     let (parsed, outline, bib) =
        //         html::post::parse_content(query.content, &sack, query.area, None);
        //     let out_buff = html::as_html(query.meta, &parsed, &sack, outline, bib);
        //     Ok(vec![(query.slug.join("index.html"), out_buff)])
        // })
        // .add_task(|sack| {
        //     Ok(vec![(
        //         "projects/index.html".into(),
        //         crate::html::to_list(
        //             &sack,
        //             sack.query_content::<Project>("projects/**/*")?
        //                 .into_iter()
        //                 .map(LinkDate::from)
        //                 .collect(),
        //             "Projects".into(),
        //             "/projects/rss.xml",
        //         ),
        //     )])
        // })
        .add_task("projects_feed", |sack| {
            let feed = crate::rss::generate_feed::<Content<Project>>(
                sack,
                "projects",
                "Kamoshi.org Projects",
            )?;
            Ok(vec![feed])
        });
});
