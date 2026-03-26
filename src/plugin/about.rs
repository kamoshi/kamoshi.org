use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Document, Image, Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;
use hypertext::prelude::*;
use minijinja::Value;
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

use crate::md::Parsed;
use crate::model::{Post, Pubkey};
use crate::props::PropsAbout;
use crate::{Context, Global};

pub fn add_about(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let docs = config
        .load_documents::<Post>()
        .source("content/about/index.md")
        .offset("content")
        .register()?;

    let cert = config
        .task()
        .glob("content/about/*.asc")
        .map(|_, _, input| {
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

    let handle = config.task().using((templates, docs, cert, images, styles)).merge(
        |ctx, (templates, docs, cert, images, styles)| {
            let document = docs.get("content/about/index.md")?;
            let pubkey_ident = cert.get("content/about/pubkey-ident.asc")?;
            let pubkey_email = cert.get("content/about/pubkey-email.asc")?;

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/page.scss")?,
            ];

            let parsed =
                crate::md::parse(&document.text, &document.meta, None, Some(&images), None)?;

            let html = render(ctx, templates, document, parsed, pubkey_ident, pubkey_email, styles)?;

            Ok(vec![
                Output::html("about", html),
                Output::binary("pubkey_ident.asc", pubkey_ident.data.clone()),
                Output::binary("pubkey_email.asc", pubkey_email.data.clone()),
            ])
        },
    );

    Ok(handle)
}

pub fn render(
    ctx: &Context,
    templates: &TemplateEnv,
    doc: &Document<Post>,
    parsed: Parsed,
    pubkey_ident: &Pubkey,
    pubkey_email: &Pubkey,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let outline_html = parsed.outline.render().into_inner();

    let props = PropsAbout {
        head: super::make_props_head(ctx, doc.matter.title.clone(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        title: doc.matter.title.clone(),
        outline: Value::from_safe_string(outline_html),
        content: Value::from_safe_string(parsed.html),
        pubkey_ident_fingerprint: pubkey_ident.fingerprint.clone(),
        pubkey_email_fingerprint: pubkey_email.fingerprint.clone(),
    };

    let tmpl = templates.get_template("about.jinja")?;
    Ok(tmpl.render(&props)?)
}
