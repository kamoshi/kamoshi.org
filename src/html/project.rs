use std::borrow::Cow;

use hauchiwa::{TaskResult, WithFile, plugin::content::Content};
use hypertext::{GlobalAttributes, Renderable, html_elements, maud_move};

use crate::{Context, html::page, model::Project};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/projects.scss"];

pub fn render_list(ctx: &Context, mut data: Vec<WithFile<Content<Project>>>) -> TaskResult<String> {
    data.sort_unstable_by(|a, b| a.data.meta.title.cmp(&b.data.meta.title));

    let main = maud_move! {
        main {
            article .project-list-wrap {
                h1 {
                    "Projects"
                }

                div .project-list-flex {
                    @for item in data {
                        (render_tile(&item.data.meta))
                    }
                }
            }
        }
    };

    let html = page(ctx, main, "Projects".into(), STYLES, Cow::default())?
        .render()
        .into_inner();

    Ok(html)
}

fn render_tile(project: &Project) -> impl Renderable {
    maud_move! {
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
