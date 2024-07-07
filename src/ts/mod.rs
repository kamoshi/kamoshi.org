mod captures;
mod configs;

use std::borrow::Cow;

use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use tree_sitter_highlight::{HighlightEvent, Highlighter};

pub enum TSEvent {
	Write(String),
	Enter(String),
	Close,
}

pub fn highlight<'data, 'html>(lang: &'data str, code: &'data str) -> impl Renderable + 'html
where
	'data: 'html,
{
	maud_move!(
		figure .listing.kanagawa data-lang=(lang) {
			pre {
				code {
					(Raw(to_html(lang, code)))
				}
			}
		}
	)
}

fn to_html(lang: &str, code: &str) -> String {
	get_events(lang, code)
		.into_iter()
		.map(|event| match event {
			TSEvent::Write(text) => Cow::from(
				text.replace('&', "&amp;")
					.replace('<', "&lt;")
					.replace('>', "&gt;"),
			),
			TSEvent::Enter(class) => {
				Cow::from(format!("<span class=\"{}\">", class.replace('.', "-")))
			}
			TSEvent::Close => Cow::from("</span>"),
		})
		.collect()
}

fn get_events(lang: &str, src: &str) -> Vec<TSEvent> {
	let config = match configs::get_config(lang) {
		Some(c) => c,
		None => return vec![TSEvent::Write(src.into())],
	};

	let mut hl = Highlighter::new();
	let highlights = hl
		.highlight(config, src.as_bytes(), None, |name| {
			configs::get_config(name)
		})
		.unwrap();

	let mut out = vec![];
	for event in highlights {
		let event = event.unwrap();
		let obj = map_event(event, src);
		out.push(obj);
	}
	out
}

fn map_event(event: HighlightEvent, src: &str) -> TSEvent {
	match event {
		HighlightEvent::Source { start, end } => TSEvent::Write(src[start..end].into()),
		HighlightEvent::HighlightStart(s) => TSEvent::Enter(captures::NAMES[s.0].into()),
		HighlightEvent::HighlightEnd => TSEvent::Close,
	}
}
