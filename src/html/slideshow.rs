use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

use crate::pipeline::{Content, Sack};
use crate::text::md::Outline;
use crate::{Link, LinkDate, Linkable};

const CSS: &str = r#"
.slides img {
	margin-left: auto;
	margin-right: auto;
	max-height: 60vh;
}
"#;

/// Represents a slideshow
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slideshow {
	pub title: String,
	#[serde(with = "super::isodate")]
	pub date: DateTime<Utc>,
	pub desc: Option<String>,
}

impl Content for Slideshow {
	fn parse(data: String, _: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
		let html = data
			.split("\n-----\n")
			.map(|chunk| {
				chunk
					.split("\n---\n")
					.map(|s| crate::text::md::parse(s.to_owned(), None))
					.map(|e| e.1)
					.collect::<Vec<_>>()
			})
			.map(|stack| match stack.len() > 1 {
				true => format!(
					"<section>{}</section>",
					stack
						.into_iter()
						.map(|slide| format!("<section>{slide}</section>"))
						.collect::<String>()
				),
				false => format!("<section>{}</section>", stack[0]),
			})
			.collect::<String>();
		(Outline(vec![]), html, None)
	}

	fn render<'s, 'p, 'html>(
		self,
		sack: &'s Sack,
		parsed: impl Renderable + 'p,
		_: Outline,
		_: Option<Vec<String>>,
	) -> impl Renderable + 'html
	where
		's: 'html,
		'p: 'html,
	{
		show(self, sack, parsed)
	}

	fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
		Some(Linkable::Date(LinkDate {
			link: Link {
				path,
				name: self.title.to_owned(),
				desc: self.desc.to_owned(),
			},
			date: self.date.to_owned(),
		}))
	}
}

pub fn show<'s, 'p, 'html>(
	fm: Slideshow,
	sack: &'s Sack,
	slides: impl Renderable + 'p,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	crate::html::bare(
		sack,
		maud_move!(
			div .reveal {
				div .slides {
					(slides)
				}
			}

			script type="module" {
				(Raw("import 'reveal';"))
			}

			style { (Raw(CSS)) }
		),
		fm.title.clone(),
	)
}
