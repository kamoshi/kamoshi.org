mod radicals;

use camino::{Utf8Path, Utf8PathBuf};
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Stylesheet}; // Document is removed from imports
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Project;
use crate::{Context, Global};

use super::make_page;

/// A local view model to decouple the renderer from the external Document struct.
/// We use references ('a) to avoid unnecessary cloning of strings.
pub struct ProjectView<'a> {
    pub title: &'a str,
    pub tech: Vec<String>,
    pub link: String,
    pub desc: Option<&'a str>,
}

pub fn build_projects(
    config: &mut Blueprint<Global>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    // Load the documents as before
    let docs = config.load_documents::<Project>("content/projects/**/*.md")?;

    let page_radicals = radicals::build(config, styles)?;

    Ok(task!(config, |ctx, docs, styles, page_radicals| {
        let styles_list = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/projects.scss")?,
        ];

        // Map the external Document<Project> to our local ProjectView
        // This effectively "strips" the external container
        let mut project_views: Vec<ProjectView> = docs
            .values()
            .map(|doc| ProjectView {
                title: &doc.metadata.title,
                tech: doc.metadata.tech.clone(),
                link: doc.metadata.link.clone(),
                desc: doc.metadata.desc.as_deref(),
            })
            .collect();

        project_views.push(ProjectView {
            title: "Radicals",
            tech: vec!["Svelte".into(), "TypeScript".into()],
            link: Utf8PathBuf::from("/")
                .join(&page_radicals.url)
                .parent()
                .unwrap()
                .to_string(),
            desc: Some("An interactive graph of related radicals and kanji."),
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
    }))
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
