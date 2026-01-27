use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Document, Image, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::{Raw, prelude::*};
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

use crate::md::Parsed;
use crate::model::{Post, Pubkey};
use crate::{Context, Global};

use super::make_page;

pub fn build_about(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let docs = config
        .load_documents::<Post>()
        .source("content/about/index.md")
        .offset("content")
        .register()?;

    let cert = config.load("content/about/*.asc", |_, _, input| {
        let data = input.read()?;

        Ok(Pubkey {
            fingerprint: Cert::from_reader(&*data)?
                .primary_key()
                .key()
                .fingerprint()
                .to_spaced_hex(),
            data: String::from_utf8(data.to_vec())?.to_string(),
        })
    })?;

    Ok(task!(config, |ctx, docs, cert, images, styles| {
        let document = docs.get("content/about/index.md")?;
        let pubkey_ident = cert.get("content/about/pubkey-ident.asc")?;
        let pubkey_email = cert.get("content/about/pubkey-email.asc")?;

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/page.scss")?,
        ];

        let parsed = crate::md::parse(&document.text, &document.meta, None, Some(images), None)?;

        let html = render(ctx, document, parsed, pubkey_ident, pubkey_email, styles)?
            .render()
            .into_inner();

        Ok(vec![
            Output::html("about", html),
            Output::binary("pubkey_ident.asc", pubkey_ident.data.clone()),
            Output::binary("pubkey_email.asc", pubkey_email.data.clone()),
        ])
    }))
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    doc: &'ctx Document<Post>,
    parsed: Parsed,
    pubkey_ident: &'ctx Pubkey,
    pubkey_email: &'ctx Pubkey,
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
        main {
            // Outline (left)
            (&parsed.outline)
            // Article (center)
            article .article {
                section .paper {
                    header {
                        h1 #top {
                            (&doc.matter.title)
                        }
                    }
                    section .wiki-article__markdown.markdown {
                        (Raw::dangerously_create(&parsed.html))

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

    make_page(ctx, main, doc.matter.title.clone(), styles, &[])
}
