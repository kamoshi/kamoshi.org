use std::borrow::Cow;

use hauchiwa::{Page, Plugin, RuntimeError, loader};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::markdown::md_parse_simple;
use crate::model::{Microblog, MicroblogEntry};
use crate::shared::make_page;
use crate::{CONTENT, Context, Global};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([loader::glob_assets(CONTENT, "twtxt.txt", |_, data| {
            let data = String::from_utf8_lossy(&data);
            let entries = data
                .lines()
                .filter(|line| {
                    let line = line.trim_start();
                    !line.is_empty() && !line.starts_with('#')
                })
                .map(str::parse::<MicroblogEntry>)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            Ok(Microblog {
                entries,
                data: data.to_string(),
            })
        })])
        .add_task("microblog", |ctx| {
            let data = ctx
                .glob::<Microblog>("twtxt.txt")?
                .into_iter()
                .next()
                .unwrap();
            let html = render(&ctx, data)?.render();

            let mut pages = vec![
                Page::text("twtxt.txt", data.data.clone()),
                Page::text("thoughts/index.html", html),
            ];

            for entry in &data.entries {
                let html = render_entry(&ctx, entry)?.render();

                pages.push(Page::html(
                    format!("thoughts/{}", entry.date.timestamp()),
                    html,
                ));
            }

            Ok(pages)
        });
});

const STYLES: &[&str] = &["styles.scss", "microblog.scss"];

pub fn render<'ctx>(
    ctx: &'ctx Context,
    microblog: &'ctx Microblog,
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let mut entries = microblog.entries.clone();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let main = maud_move!(
        main {
            section .microblog {
                @for entry in &entries {
                    article {
                        a href=(format!("/thoughts/{}/", entry.date.timestamp())) {
                            time datetime=(entry.date.to_rfc3339()) {
                                (entry.date.format("%Y-%m-%d %H:%M UTC").to_string())
                            }
                        }
                        (Raw(md_parse_simple(&entry.text)))
                    }
                }
            }
        }
    );

    make_page(ctx, main, "microblog".into(), STYLES, Cow::default())
}

pub fn render_entry<'ctx>(
    ctx: &'ctx Context,
    entry: &'ctx MicroblogEntry,
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud_move!(
        main {
            section .microblog {
                article {
                    time datetime=(entry.date.to_rfc3339()) {
                        (entry.date.format("%Y-%m-%d %H:%M UTC").to_string())
                    }
                    (Raw(md_parse_simple(&entry.text)))
                }
            }
        }
    );

    make_page(ctx, main, "microblog".into(), STYLES, Cow::default())
}
