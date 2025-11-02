use hauchiwa::error::RuntimeError;
use hauchiwa::loader::{self, CSS, Content, Registry, glob_content};
use hauchiwa::page::Page;
use hauchiwa::task::Handle;
use hauchiwa::{SiteConfig, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::{Post, Project};
use crate::{CONTENT, Context, Global};

use super::make_page;

pub fn build_projects(
    config: &mut SiteConfig<Global>,
    styles: Handle<Registry<CSS>>,
) -> Handle<Vec<Page>> {
    let projects = glob_content::<_, Project>(config, "content/projects/**/*.md");

    config.add_task((projects, styles), |ctx, (projects, styles)| {
        let mut pages = vec![];

        let styles_list = &[
            styles.get("styles/styles.scss").unwrap(),
            styles.get("styles/layouts/projects.scss").unwrap(),
        ];

        // let styles_page = &[
        //     styles.get("styles/styles.scss").unwrap(),
        //     styles.get("styles/layouts/page.scss").unwrap(),
        // ];

        {
            let data = projects.values().collect::<Vec<_>>();
            let list = render_list(&ctx, data, styles_list).unwrap();
            pages.push(Page::html("projects", list));

            // let text = ctx.get::<String>("hauchiwa")?;
            // let article = crate::markdown::parse(&ctx, text, "".into(), None)?;
            // let html = render_page(&ctx, &article)?.render();
            // pages.push(Page::html("projects/hauchiwa", html));
        }

        {
            //             let feed = crate::rss::generate_feed::<Content<Project>>(
            //                 sack,
            //                 "projects",
            //                 "Kamoshi.org Projects",
            //             )?;
            //             Ok(vec![feed])
        }

        pages
    })
}

pub fn render_list(
    ctx: &Context,
    mut data: Vec<&Content<Project>>,
    styles: &[&CSS],
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
    styles: &'ctx [&CSS],
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

    let html = make_page(ctx, html, "".into(), styles, &[])?;

    Ok(html)
}
