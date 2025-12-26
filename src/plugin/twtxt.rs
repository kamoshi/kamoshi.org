use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Registry, Stylesheet};
use hauchiwa::page::Page;
use hauchiwa::task::Handle;
use hauchiwa::{SiteConfig, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::md_parse_simple;
use crate::model::{Microblog, MicroblogEntry};
use crate::{Context, Global};

use super::make_page;

pub fn build_twtxt(
    config: &mut SiteConfig<Global>,
    styles: Handle<Registry<Stylesheet>>,
) -> Result<Handle<Vec<Page>>, HauchiwaError> {
    let twtxt = config.load("content/twtxt.txt", |_, _, file| {
        let data = String::from_utf8_lossy(&file.data);
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
    })?;

    Ok(task!(config, |ctx, twtxt, styles| {
        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/microblog.scss")?,
        ];

        let data = twtxt.get("content/twtxt.txt")?;
        let html = render(ctx, data, styles)?.render();

        let mut pages = vec![
            Page::file("twtxt.txt", data.data.clone()),
            Page::html("thoughts", html),
        ];

        for entry in &data.entries {
            let html = render_entry(ctx, entry, styles)?.render();

            pages.push(Page::html(
                format!("thoughts/{}", entry.date.timestamp()),
                html.into_inner(),
            ));
        }

        Ok(pages)
    }))
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    microblog: &'ctx Microblog,
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let mut entries = microblog.entries.clone();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let main = maud!(
        main {
            section .microblog {
                @for entry in &entries {
                    (render_tweet(entry))
                }
            }
        }
    );

    make_page(ctx, main, "microblog".into(), styles, &[])
}

pub fn render_entry<'ctx>(
    ctx: &'ctx Context,
    entry: &'ctx MicroblogEntry,
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
        main {
            section .microblog {
                (render_tweet(entry))
            }
        }
    );

    make_page(ctx, main, "microblog".into(), styles, &[])
}

fn render_tweet(entry: &MicroblogEntry) -> impl Renderable {
    maud!(
        article .tweet {
            // Left Column: Avatar
            div .tweet-avatar {
                // Using a placeholder service. Replace src with your user's PFP url.
                img src="/aya_shades.png" alt="Avatar";
            }

            // Right Column: Header + Content
            div .tweet-content {
                header .tweet-header {
                    span .display-name { "kamov" }
                    span .handle { "@kamov" }
                    span .separator { "·" }
                    a .tweet-link href=(format!("/thoughts/{}/", entry.date.timestamp())) {
                        time datetime=(entry.date.to_rfc3339()) {
                            (entry.date.format("%b %d").to_string())
                        }
                    }
                }

                div .tweet-body {
                    (Raw(md_parse_simple(&entry.text)))
                }
            }
        }
    )
}
