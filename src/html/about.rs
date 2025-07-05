use std::borrow::Cow;

use hauchiwa::{TaskResult, ViewPage};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::{
    Context, Outline,
    html::post::render_metadata,
    model::{Post, Pubkey},
};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/page.scss"];

pub fn render<'s, 'p, 'html>(
    ctx: &'s Context,
    item: &'p ViewPage<Post>,
    parsed: String,
    outline: Outline,
    pubkey_ident: &'p ViewPage<Pubkey>,
    pubkey_email: &'p ViewPage<Pubkey>,
) -> TaskResult<impl Renderable + use<'html>>
where
    's: 'html,
    'p: 'html,
{
    let main = maud_move!(
        main {
            // Outline (left)
            (render_outline(outline))
            // Article (center)
            article .article {
                section .paper {
                    header {
                        h1 #top {
                            (&item.meta.title)
                        }
                    }
                    section .wiki-article__markdown.markdown {
                        (Raw(parsed))

                        h2 {
                           "Keys"
                        }
                        p {
                            "GPG public key"
                        }
                        a href="/pubkey_ident.asc" {
                            (&pubkey_ident.meta.fingerprint)
                        }
                        p {
                            "GPG public key (email)"
                        }
                        a href="/pubkey_email.asc" {
                            (&pubkey_email.meta.fingerprint)
                        }
                    }
                }
            }
            // Metadata (right)
            (render_metadata(ctx, item.meta, item.info))
        }
    );

    crate::html::page(ctx, main, item.meta.title.clone(), STYLES, Cow::default())
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
                                a href=(format!("#{id}")) {
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
