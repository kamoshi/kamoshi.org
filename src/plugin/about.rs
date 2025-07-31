use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin, RuntimeError, WithFile};
use hypertext::{Raw, prelude::*};
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

use crate::markdown::Article;
use crate::model::{Post, Pubkey};
use crate::{CONTENT, Context, Global};

use super::make_page;
use super::posts::render_metadata;

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            loader::glob_content(CONTENT, "about/index.md", yaml::<Post>),
            // .asc -> Pubkey
            loader::glob_assets(CONTENT, "about/*.asc", |_, data| {
                Ok(Pubkey {
                    fingerprint: Cert::from_reader(data.as_slice())?
                        .primary_key()
                        .key()
                        .fingerprint()
                        .to_spaced_hex(),
                    data: String::from_utf8(data)?.to_string(),
                })
            }),
        ])
        .add_task("about", |ctx| {
            let item = ctx.glob_one_with_file::<Content<Post>>("about.md")?;
            let pubkey_ident = ctx.get::<Pubkey>("about/pubkey-ident.asc")?;
            let pubkey_email = ctx.get::<Pubkey>("about/pubkey-email.asc")?;

            let article = crate::markdown::parse(&ctx, &item.data.text, &item.file.area, None)?;
            let html = render(&ctx, &item, article, pubkey_ident, pubkey_email)?.render();

            let pages = vec![
                Page::html_with_file(&item.file.area, html, item.file.clone()),
                Page::text("pubkey_ident.asc", pubkey_ident.data.clone()),
                Page::text("pubkey_email.asc", pubkey_email.data.clone()),
            ];

            Ok(pages)
        });
});

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "layouts/page.scss"];

pub fn render<'ctx>(
    ctx: &'ctx Context,
    item: &'ctx WithFile<Content<Post>>,
    article: Article,
    pubkey_ident: &'ctx Pubkey,
    pubkey_email: &'ctx Pubkey,
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
        main {
            // Outline (left)
            (&article.outline)
            // Article (center)
            article .article {
                section .paper {
                    header {
                        h1 #top {
                            (&item.data.meta.title)
                        }
                    }
                    section .wiki-article__markdown.markdown {
                        (Raw(&article.text))

                        h2 {
                           "Keys"
                        }
                        p {
                            "GPG public key"
                        }
                        a href="/pubkey_ident.asc" {
                            (&pubkey_ident.fingerprint)
                        }
                        p {
                            "GPG public key (email)"
                        }
                        a href="/pubkey_email.asc" {
                            (&pubkey_email.fingerprint)
                        }
                    }
                }
            }
            // Metadata (right)
            (render_metadata(ctx, &item.data.meta, item.file.info.as_ref(), &[]))
        }
    );

    make_page(
        ctx,
        main,
        item.data.meta.title.clone(),
        STYLES,
        Default::default(),
    )
}
