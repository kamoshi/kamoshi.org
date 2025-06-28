use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::{Context, md::md_parse_simple, model::Microblog};

const STYLES: &[&str] = &["styles/styles.scss", "styles/microblog.scss"];

pub fn render<'a>(
    ctx: &'a Context,
    microblog: &'a Microblog,
) -> TaskResult<impl Renderable + use<'a>> {
    let mut entries = microblog.entries.clone();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let main = maud_move!(
        main {
            section .microblog {
                @for entry in &entries {
                    article {
                        time datetime=(entry.date.to_rfc3339()) {
                            (entry.date.format("%Y-%m-%d %H:%M UTC").to_string())
                        }
                        (Raw(md_parse_simple(&entry.text)))
                    }
                }
            }
        }
    );

    crate::html::page(ctx, main, "microblog".into(), STYLES, None)
}
