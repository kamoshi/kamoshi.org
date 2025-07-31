use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin, RuntimeError, WithFile};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Project;
use crate::{CONTENT, Context, Global};

use super::make_page;

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "projects/**/*.md", yaml::<Project>),
        ])
        .add_task("projects", |ctx| {
            let mut pages = vec![];

            let data = ctx.glob_with_file::<Content<Project>>("projects/**/*")?;
            let list = render_list(&ctx, data)?;
            pages.push(Page::html("projects", list));

            let text = ctx.get::<String>("hauchiwa")?;
            let article = crate::markdown::parse(&ctx, text, "".into(), None)?;
            let html = render_page(&ctx, &article)?.render();
            pages.push(Page::html("projects/hauchiwa", html));

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

const STYLES_LIST: &[&str] = &["styles.scss", "layouts/projects.scss"];
const STYLES_PAGE: &[&str] = &["styles.scss", "layouts/page.scss"];

pub fn render_list(
    ctx: &Context,
    mut data: Vec<WithFile<Content<Project>>>,
) -> Result<String, RuntimeError> {
    data.sort_unstable_by(|a, b| a.data.meta.title.cmp(&b.data.meta.title));

    let main = maud! {
        main {
            article .project-list-wrap {
                h1 {
                    "Projects"
                }

                div .project-list-flex {
                    @for item in &data {
                        (render_tile(&item.data.meta))
                    }
                }
            }
        }
    };

    let html = make_page(
        ctx,
        main,
        "Projects".into(),
        STYLES_LIST,
        Default::default(),
    )?
    .render()
    .into_inner();

    Ok(html)
}

fn render_tile(project: &Project) -> impl Renderable {
    maud! {
        a .project-list-tile href=(&project.link) {
            h2 { (&project.title) }
            ul .tech-stack {
                @for tech in &project.tech {
                    li {
                        img src=(format!("/static/svg/tech/{}.svg", tech.to_lowercase())) alt=(tech);
                    }
                }
            }
            @if let Some(desc) = &project.desc {
                p { (desc) }
            }
        }
    }
}

pub fn render_page<'ctx>(
    ctx: &'ctx Context,
    article: &'ctx Article,
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let html = maud!(
        main {
            // Outline (left)
            (&article.outline)
            // Article (center)
            article .article {
                section .paper {
                    section .wiki-article__markdown.markdown {
                        (Raw(&article.text))
                    }
                }
            }
            // Metadata (right)
            aside .tiles {
                section .metadata {
                }
            }
        }
    );

    let html = make_page(ctx, html, "".into(), STYLES_PAGE, Default::default())?;

    Ok(html)
}
