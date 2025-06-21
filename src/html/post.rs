use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::model::Post;
use crate::{Bibliography, Context, Outline};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/page.scss"];

pub fn render<'s, 'p, 'html>(
    meta: &'p Post,
    parsed: &'p str,
    ctx: &'s Context,
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
    ctx: &'s Context,
    meta: &'p Post,
    parsed: &'p str,
    info: Option<&hauchiwa::GitInfo>,
    outline: Outline,
    bibliography: Bibliography,
) -> impl Renderable {
    maud_move!(
        // Outline (left)
        (render_outline(outline))
        // Article (center)
        (render_article(meta, parsed, bibliography))
        // Metadata (right)
        (render_metadata(ctx, meta, info))
    )
}

fn render_outline(outline: Outline) -> impl Renderable {
    maud_move!(
        aside .outline {
            section {
                h2 {
                    a href="#top" { "Outline" }
                }
                nav #table-of-contents {
                    ul {
                        @for (title, id) in outline.0 {
                            li {
                                a href=(format!("#{}", id)) {
                                    (title)
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}

fn render_article(meta: &Post, parsed: &str, bib: Bibliography) -> impl Renderable {
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
    )
}

fn render_metadata(
    ctx: &Context,
    meta: &Post,
    info: Option<&hauchiwa::GitInfo>,
) -> impl Renderable {
    maud_move!(
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
                        a href=(format!("{}/commit/{}", &ctx.get_globals().data.link, &info.abbreviated_hash)) {
                            (&info.abbreviated_hash)
                        }
                    }
                }
            }
        }
    )
}
