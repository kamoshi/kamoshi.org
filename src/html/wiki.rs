use std::borrow::Cow;

use camino::Utf8Path;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::{Bibliography, Context, Outline, model::Wiki};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "layouts/page.scss"];

pub fn wiki(
    meta: &Wiki,
    parsed: &str,
    ctx: &Context,
    slug: &Utf8Path,
    _: Outline,
    bib: &Bibliography,
    library_path: Option<&Utf8Path>,
) -> String {
    let main = maud_move!(
        main .wiki-main {
            // Outline
            (render_outline(ctx, slug))
            // Article
            (render_article(meta, parsed, bib, library_path))
        }
    );

    crate::html::page(ctx, main, meta.title.to_owned(), STYLES, Cow::default())
        .unwrap()
        .render()
        .into()
}

fn render_outline(ctx: &Context, slug: &Utf8Path) -> impl Renderable {
    maud_move!(
        aside .outline {
            section {
                div {
                    (crate::html::misc::show_page_tree(slug, ctx))
                }
            }
        }
    )
}

fn render_article(
    meta: &Wiki,
    parsed: &str,
    bib: &Bibliography,
    library_path: Option<&Utf8Path>,
) -> impl Renderable {
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

            @if let Some(bib) = &bib.0 {
                (crate::html::misc::emit_bibliography(bib, library_path))
            }
        }
    )
}
