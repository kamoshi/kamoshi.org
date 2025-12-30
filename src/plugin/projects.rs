use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Document, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Project;
use crate::{Context, Global};

use super::make_page;

pub fn build_projects(
    config: &mut Blueprint<Global>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let docs = config.load_documents::<Project>("content/projects/**/*.md")?;

    Ok(task!(config, |ctx, docs, styles| {
        let docs = docs.values().collect::<Vec<_>>();

        let styles_list = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/projects.scss")?,
        ];

        // let styles_page = &[
        //     styles.get("styles/styles.scss").unwrap(),
        //     styles.get("styles/layouts/page.scss").unwrap(),
        // ];

        let mut pages = vec![];

        {
            pages.push(crate::rss::generate_feed(
                &docs,
                "projects",
                "Kamoshi.org Projects",
            ));
        }

        {
            let list = render_list(ctx, docs, styles_list)?;
            pages.push(Output::html("projects", list));

            // let text = ctx.get::<String>("hauchiwa")?;
            // let article = crate::markdown::parse(&ctx, text, "".into(), None)?;
            // let html = render_page(&ctx, &article)?.render();
            // pages.push(Page::html("projects/hauchiwa", html));
        }

        Ok(pages)
    }))
}

pub fn render_list(
    ctx: &Context,
    mut data: Vec<&Document<Project>>,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    data.sort_unstable_by(|a, b| a.metadata.title.cmp(&b.metadata.title));

    let main = maud! {
        main {
            article .project-list-wrap {
                h1 {
                    "Projects"
                }

                div .project-list-flex {
                    @for item in &data {
                        (render_tile(&item.metadata))
                    }
                }
            }
        }
    };

    let html = make_page(ctx, main, "Projects".into(), styles, &[])?
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
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let html = maud!(
        main {
            // Outline (left)
            (&article.outline)
            // Article (center)
            article .article {
                section .paper {
                    section .wiki-article__markdown.markdown {
                        (Raw::dangerously_create(&article.text))
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

    let html = make_page(ctx, html, "".into(), styles, &[])?;

    Ok(html)
}
