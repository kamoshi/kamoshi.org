use hauchiwa::error::RuntimeError;
use hauchiwa::loader::{CSS, Content, Image, Registry, glob_assets, glob_content};
use hauchiwa::page::Page;
use hauchiwa::task::Handle;
use hauchiwa::{SiteConfig, task};
use hypertext::{Raw, prelude::*};
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

use crate::markdown::Article;
use crate::model::{Post, Pubkey};
use crate::{Context, Global};

use super::make_page;

pub fn build_about(
    site_config: &mut SiteConfig<Global>,
    images: Handle<Registry<Image>>,
    styles: Handle<Registry<CSS>>,
) -> Handle<Vec<Page>> {
    let page = glob_content::<_, Post>(site_config, "content/about/index.md");

    let cert = glob_assets(site_config, "content/about/*.asc", |_, file| {
        Ok(Pubkey {
            fingerprint: Cert::from_reader(file.metadata.as_slice())?
                .primary_key()
                .key()
                .fingerprint()
                .to_spaced_hex(),
            data: String::from_utf8(file.metadata)?.to_string(),
        })
    });

    task!(site_config, |ctx, page, cert, images, styles| {
        let item = page.get("content/about/index.md")?;
        let pubkey_ident = cert.get("content/about/pubkey-ident.asc")?;
        let pubkey_email = cert.get("content/about/pubkey-email.asc")?;

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/page.scss")?,
        ];

        let article = crate::markdown::parse(&item.content, &item.path, None, Some(images))?;
        let html = render(&ctx, &item, article, pubkey_ident, pubkey_email, styles)?.render();

        let pages = vec![
            Page::html("about", html),
            Page::file("pubkey_ident.asc", pubkey_ident.data.clone()),
            Page::file("pubkey_email.asc", pubkey_email.data.clone()),
        ];

        Ok(pages)
    });
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    item: &'ctx Content<Post>,
    article: Article,
    pubkey_ident: &'ctx Pubkey,
    pubkey_email: &'ctx Pubkey,
    styles: &'ctx [&CSS],
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
                            (&item.metadata.title)
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
            // (render_metadata(ctx, &item.data.meta, item.file.info.as_ref(), &[]))
        }
    );

    make_page(ctx, main, item.metadata.title.clone(), styles, &[])
}
