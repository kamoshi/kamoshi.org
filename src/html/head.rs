use hauchiwa::{Mode, TaskResult};
use hypertext::{Raw, Renderable, html_elements, maud_move};

use crate::Context;

const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", event => {
	window.location.reload();
});
"#;

pub(crate) fn render_head<'a>(
    ctx: &'a Context,
    title: String,
    stylesheets: &'a [&str],
    script: Option<&'a [String]>,
) -> TaskResult<impl Renderable> {
    let globals = ctx.get_globals();
    let title = format!("{} | kamoshi.org", title);

    let stylesheets: Vec<_> = stylesheets
        .into_iter()
        .map(|&style| ctx.get_styles(style.into()))
        .collect::<Result<_, _>>()?;

    let script = match script {
        Some(script) => Some(emit_tags_script(ctx, script)?),
        None => None,
    };

    let html = maud_move!(
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";

        title { (title) }

        link rel="sitemap" href="/sitemap.xml";

        link rel="preconnect" href="https://rsms.me/";
        link rel="stylesheet" href="https://rsms.me/inter/inter.css";

        @for path in stylesheets {
            (render_tag_style(path.as_str()))
        }

        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";

        @if matches!(globals.mode, Mode::Watch) {
            script { (Raw(JS_RELOAD)) }
        }

        @if let Some(scripts) = script {
            (scripts)
        }
    );

    Ok(html)
}

fn render_tag_style(path: &str) -> impl Renderable {
    maud_move!(link rel="stylesheet" href=(path);)
}

fn emit_tags_script(sack: &Context, scripts: &[String]) -> TaskResult<impl Renderable> {
    let tags: Vec<_> = scripts
        .iter()
        .map(|script| emit_tag_module(sack, script))
        .collect::<Result<_, _>>()?;

    let html = maud_move!(
        @for tag in tags {
            (tag)
        }
    );

    Ok(html)
}

fn emit_tag_module(sack: &Context, alias: &str) -> TaskResult<impl Renderable> {
    let path = sack.get_script(alias)?;
    let html = maud_move!(script type="module" src=(path.as_str()) defer {});

    Ok(html)
}
