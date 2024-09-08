use hauchiwa::{HashedStyle, Mode, Sack};
use hypertext::{html_elements, maud_move, Raw, Renderable};

const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", (event) => {
	console.log(event);
	window.location.reload();
});
"#;

pub(crate) fn render_head<'s, 'r>(
	sack: &'s Sack,
	title: String,
	styles: &'s [&str],
	js: Option<&'s [String]>,
) -> impl Renderable + 'r
where
	's: 'r,
{
	let title = format!("{} | kamoshi.org", title);
	let css = sack.get_style("styles").expect("Missing styles");
	let css_r = sack.get_style("reveal").expect("Missing styles");
	let css_p = sack.get_style("leaflet").expect("Missing styles");

	maud_move!(
		meta charset="utf-8";
		meta name="viewport" content="width=device-width, initial-scale=1";
		title {
			(title)
		}

		// link rel="sitemap" href="/sitemap.xml";

		(render_style(css))
		(render_style(css_r))
		(render_style(css_p))

		link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
		link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
		link rel="icon" href="/favicon.ico" sizes="any";

		script type="importmap" {(Raw(sack.get_import_map()))}

		@if matches!(sack.ctx.mode, Mode::Watch) {
			script { (Raw(JS_RELOAD)) }
		}

		@if let Some(scripts) = js {
		    @for script in scripts {
				(emit_tag_script(sack, script))
			}
		}
	)
}

fn render_style(style: &HashedStyle) -> impl Renderable + '_ {
	maud_move!(
		link rel="stylesheet" href=(style.path.as_str());
	)
}

fn emit_tag_script<'a>(sack: &'a Sack, script: &'a str) -> impl Renderable + 'a {
    let src = sack.get_script(script).unwrap().path.as_str();

    maud_move!(
        script type="module" src=(src) defer {}
    )
}
