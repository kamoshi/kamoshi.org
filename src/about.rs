use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin};
use hypertext::Renderable as _;
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

use crate::model::{Post, Pubkey};
use crate::{CONTENT, Global};

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

            let (parsed, outline, _) =
                crate::md::parse(&ctx, &item.data.text, &item.file.area, None)?;
            let html = crate::html::about::render(
                &ctx,
                &item,
                &parsed,
                &outline,
                pubkey_ident,
                pubkey_email,
            )?
            .render()
            .into();

            let pages = vec![
                Page::text("about/index.html".into(), html),
                Page::text("pubkey_ident.asc".into(), pubkey_ident.data.to_owned()),
                Page::text("pubkey_email.asc".into(), pubkey_email.data.to_owned()),
            ];

            Ok(pages)
        });
});
