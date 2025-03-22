use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::model::Post;
use crate::{Bibliography, MySack, Outline};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/page.scss"];

pub fn render<'s, 'p, 'html>(
    meta: &'p Post,
    parsed: &'p str,
    ctx: &'s MySack,
    info: Option<&'s hauchiwa::GitInfo>,
    outline: Outline,
    bibliography: Bibliography,
) -> TaskResult<impl Renderable + use<'html>>
where
    's: 'html,
    'p: 'html,
{
    let main = maud_move!(
        main {
            (article(ctx, meta, parsed, info, outline, bibliography))
        }
    );

    crate::html::page(
        ctx,
        main,
        meta.title.clone(),
        STYLES,
        meta.scripts.as_deref(),
    )
}

pub fn article<'p, 's>(
    ctx: &'s MySack,
    meta: &'p Post,
    parsed: &'p str,
    info: Option<&hauchiwa::GitInfo>,
    outline: Outline,
    bibliography: Bibliography,
) -> impl Renderable {
    maud_move!(
        // Slide in/out for mobile
        input #wiki-aside-shown type="checkbox" hidden;

        aside .outline {
            // Slide button
            (crate::html::misc::show_outline(outline))
        }

        (paper_page(ctx, meta, info, parsed, bibliography))
    )
}

fn paper_page<'a>(
    ctx: &'a MySack,
    meta: &'a Post,
    info: Option<&'a hauchiwa::GitInfo>,
    parsed: &'a str,
    bib: Bibliography,
) -> impl Renderable + 'a {
    maud_move!(
        article .article {
            section .paper {
                header {
                    h1 #top {
                        (&meta.title)
                    }
                }
                section .wiki-article__markdown.markdown {
                    (Raw(parsed))
                }
            }

            @if let Some(bib) = bib.0 {
                (crate::html::misc::emit_bibliography(bib))
            }
        }

        aside .tiles {
            section .metadata {
                h2 {
                    "Metadata"
                }
                div {
                    img src="/static/svg/icon_add.svg" title="Added";
                    time datetime=(meta.date.format("%Y-%m-%d").to_string()) {
                        (meta.date.format("%Y, %B %d").to_string())
                    }
                }
                @if let Some(info) = info {
                    div {
                        img src="/static/svg/icon_update.svg" title="Updated";
                        time datetime=(info.commit_date.format("%Y-%m-%d").to_string()) {
                            (info.commit_date.format("%Y, %B %d").to_string())
                        }
                    }
                    div {
                        img src="/static/svg/icon_link.svg" title="Link to commit";
                        a href=(format!("{}/commit/{}", &ctx.get_metadata().data.link, &info.abbreviated_hash)) {
                            (&info.abbreviated_hash)
                        }
                    }
                }
            }
        }
    )
}
