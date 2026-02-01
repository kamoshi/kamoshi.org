mod radicals;

use camino::Utf8PathBuf;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::Stylesheet;
use hauchiwa::prelude::*;
use hypertext::{Raw, prelude::*};

use crate::md::Parsed;
use crate::model::Project;
use crate::{Context, Global};

use super::make_page;

pub struct ProjectView<'a> {
    pub title: &'a str,
    pub tech: Vec<String>,
    pub link: String,
    pub desc: Option<&'a str>,
}

pub fn add_projects(
    config: &mut Blueprint<Global>,
    styles: Many<Stylesheet>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let docs = config
        .load_documents::<Project>()
        .source("content/projects/**/*.md")
        .offset("content")
        .register()?;

    let page_radicals = radicals::build(config, styles)?;

    let task = config.task().depends_on((docs, styles, page_radicals)).run(
        |ctx, (docs, styles, page_radicals)| {
            let styles_list = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/projects.scss")?,
            ];

            // Map the external Document<Project> to our local ProjectView
            // This effectively "strips" the external container
            let mut project_views: Vec<ProjectView> = docs
                .values()
                .map(|doc| ProjectView {
                    title: &doc.matter.title,
                    tech: doc.matter.tech.clone(),
                    link: doc.matter.link.clone(),
                    desc: doc.matter.desc.as_deref(),
                })
                .collect();

            project_views.push(ProjectView {
                title: "Constellations",
                tech: vec!["Svelte".into(), "TypeScript".into()],
                link: Utf8PathBuf::from("/")
                    .join(&page_radicals.path)
                    .parent()
                    .unwrap()
                    .to_string(),
                desc: Some("Try adding kanji you know and see how they connect to each other."),
            });

            let mut pages = vec![];

            // Note: crate::rss::generate_feed likely still needs the original docs
            // or a similar transformation depending on its signature.
            // Assuming it keeps working with references to the raw map:
            {
                let docs_vec = docs.values().collect::<Vec<_>>();
                pages.push(crate::rss::generate_feed(
                    &docs_vec,
                    "projects",
                    "Kamoshi.org Projects",
                ));
            }

            {
                // Pass the mapped views instead of the docs
                let list = render_list(ctx, project_views, styles_list)?;
                pages.push(Output::html("projects", list));
            }

            // Removed `pages.push(value);` as `value` was undefined in snippet

            Ok(pages)
        },
    );

    Ok(task)
}

pub fn render_list(
    ctx: &Context,
    mut projects: Vec<ProjectView>, // Now accepts our local struct
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    // Sort logic remains the same, but operates on the view struct
    projects.sort_unstable_by(|a, b| a.title.cmp(b.title));

    let main = maud! {
        main {
            article .project-list-wrap {
                h1 {
                    "Projects"
                }

                div .project-list-flex {
                    @for item in &projects {
                        (render_tile(item))
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

// Updated to take the ProjectView
fn render_tile(project: &ProjectView) -> impl Renderable {
    maud! {
        a .project-list-tile href=(project.link) {
            h2 { (project.title) }
            ul .tech-stack {
                @for tech in &project.tech {
                    li {
                        img src=(format!("/static/svg/tech/{}.svg", tech.to_lowercase())) alt=(tech);
                    }
                }
            }
            @if let Some(desc) = project.desc {
                p { (desc) }
            }
        }
    }
}

pub fn render_page<'ctx>(
    ctx: &'ctx Context,
    parsed: &'ctx Parsed,
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let html = maud!(
        main {
            // Outline (left)
            (&parsed.outline)
            // Article (center)
            article .article {
                section .paper {
                    section .wiki-article__markdown.markdown {
                        (Raw::dangerously_create(&parsed.html))
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
