use std::borrow::Cow;

use hauchiwa::{Stylesheet, TaskResult};
use hypertext::{Raw, Renderable, html_elements, maud_move};

use crate::Context;

pub(crate) fn render_head<'a>(
    ctx: &'a Context,
    title: String,
    stylesheets: &'a [&str],
    script: Cow<'a, [String]>,
) -> TaskResult<impl Renderable> {
    let title = format!("{title} | kamoshi.org");

    let stylesheets: Vec<_> = stylesheets
        .iter()
        .map(|&style| ctx.get::<Stylesheet>(style))
        .collect::<Result<_, _>>()?;

    Ok(maud_move!(
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";

        title { (title) }

        link rel="sitemap" href="/sitemap.xml";

        link rel="preconnect" href="https://rsms.me/";
        link rel="stylesheet" href="https://rsms.me/inter/inter.css";

        @for stylesheet in stylesheets {
            link rel="stylesheet" href=(stylesheet.path.as_str());
        }

        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";

        @for path in script.as_ref() {
            script type="module" src=(path) defer {}
        }

        @if let Some(reload_script) = ctx.get_refresh_script() {
            script { (Raw(reload_script)) }
        }
    ))
}
