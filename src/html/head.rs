use hauchiwa::{Mode, Sack, TaskResult};
use hypertext::{Raw, Renderable, html_elements, maud_move};

use crate::Global;

const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", event => {
	window.location.reload();
});
"#;

pub(crate) fn render_head<'a>(
    ctx: &'a Sack<Global>,
    title: String,
    styles: &'a [&str],
    script: Option<&'a [String]>,
) -> TaskResult<impl Renderable> {
    let metadata = ctx.get_metadata();
    let title = format!("{} | kamoshi.org", title);

    let css_s = ctx.get_styles("styles/styles.scss".into())?;
    let css_r = ctx.get_styles("styles/reveal/reveal.scss".into())?;
    let css_p = ctx.get_styles("styles/photos/leaflet.scss".into())?;

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

        (render_tag_style(css_s.as_str()))
        (render_tag_style(css_r.as_str()))
        (render_tag_style(css_p.as_str()))

        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";

        @if matches!(metadata.mode, Mode::Watch) {
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

fn emit_tags_script(sack: &Sack<Global>, scripts: &[String]) -> TaskResult<impl Renderable> {
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

fn emit_tag_module(sack: &Sack<Global>, alias: &str) -> TaskResult<impl Renderable> {
    let path = sack.get_script(alias)?;
    let html = maud_move!(script type="module" src=(path.as_str()) defer {});

    Ok(html)
}
