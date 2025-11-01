use hauchiwa::SiteConfig;
use hauchiwa::error::RuntimeError;
use hauchiwa::loader::{CSS, Registry, glob_assets};
use hauchiwa::page::Page;
use hauchiwa::task::Handle;
use hypertext::{Raw, prelude::*};

use crate::markdown::md_parse_simple;
use crate::model::{Microblog, MicroblogEntry};
use crate::{Context, Global};

use super::make_page;

pub fn build_twtxt(
    config: &mut SiteConfig<Global>,
    styles: Handle<Registry<CSS>>,
) -> Handle<Vec<Page>> {
    let twtxt = glob_assets(config, "content/twtxt.txt", |_, file| {
        let data = String::from_utf8_lossy(&file.metadata);
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
    });

    config.add_task((twtxt, styles), |ctx, (twtxt, styles)| {
        let styles = &[
            styles.get("styles/styles.scss").unwrap(),
            styles.get("styles/microblog.scss").unwrap(),
        ];

        let data = twtxt.get("content/twtxt.txt").unwrap();
        let html = render(&ctx, data, styles).unwrap().render();

        let mut pages = vec![
            Page {
                url: "twtxt.txt".into(),
                content: data.data.clone(),
            },
            Page {
                url: "thoughts/index.html".into(),
                content: html.into_inner(),
            },
        ];

        for entry in &data.entries {
            let html = render_entry(&ctx, entry, styles).unwrap().render();

            pages.push(Page {
                url: format!("thoughts/{}", entry.date.timestamp()),
                content: html.into_inner(),
            });
        }

        pages
    })
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    microblog: &'ctx Microblog,
    styles: &'ctx [&CSS],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let mut entries = microblog.entries.clone();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let main = maud!(
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

    make_page(ctx, main, "microblog".into(), styles, &[])
}

pub fn render_entry<'ctx>(
    ctx: &'ctx Context,
    entry: &'ctx MicroblogEntry,
    styles: &'ctx [&CSS],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
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

    make_page(ctx, main, "microblog".into(), styles, &[])
}
