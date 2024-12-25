use hauchiwa::{Mode, Sack};
use hypertext::{html_elements, maud_move, Raw, Renderable};

use crate::MyData;

const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", event => {
	window.location.reload();
});
"#;

pub(crate) fn render_head<'s, 'r>(
	sack: &'s Sack<MyData>,
	title: String,
	_styles: &'s [&str],
	scripts: Option<&'s [String]>,
) -> Result<impl Renderable + 'r, String>
where
	's: 'r,
{
	let context = sack.get_context();
	let title = format!("{} | kamoshi.org", title);
	let css = sack
		.get_styles("styles/styles.scss".into())
		.expect("Missing styles");
	let css_r = sack
		.get_styles("styles/reveal/reveal.scss".into())
		.expect("Missing styles");
	let css_p = sack
		.get_styles("styles/photos/leaflet.scss".into())
		.expect("Missing styles");

	let scripts = match scripts {
		Some(scripts) => Some(emit_tags_script(sack, scripts)?),
		None => None,
	};

	Ok(maud_move!(
		meta charset="utf-8";
		meta name="viewport" content="width=device-width, initial-scale=1";

		title {
			(title)
		}

		link rel="preconnect" href="https://rsms.me/";
		link rel="stylesheet" href="https://rsms.me/inter/inter.css";

		// link rel="sitemap" href="/sitemap.xml";

		(render_style(css.as_str()))
		(render_style(css_r.as_str()))
		(render_style(css_p.as_str()))

		link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
		link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
		link rel="icon" href="/favicon.ico" sizes="any";

		@if matches!(context.mode, Mode::Watch) {
			script { (Raw(JS_RELOAD)) }
		}

		@if let Some(scripts) = scripts {
			(scripts)
		}
	))
}

fn render_style(path: &str) -> impl Renderable + '_ {
	maud_move!(
		link rel="stylesheet" href=(path);
	)
}

fn emit_tags_script<'a>(
	sack: &'a Sack<MyData>,
	scripts: &'a [String],
) -> Result<impl Renderable + 'a, String> {
	let tags = scripts
		.iter()
		.map(|script| emit_tag_script(sack, script))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(maud_move!(
		@for tag in tags {
			(tag)
		}
	))
}

fn emit_tag_script<'a>(
	sack: &'a Sack<MyData>,
	alias: &'a str,
) -> Result<impl Renderable + 'a, String> {
	let path = sack
		.get_script(alias)
		.ok_or(format!("Missing script {}", alias))?;

	Ok(maud_move!(script type="module" src=(path.as_str()) defer {}))
}
