use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use hauchiwa::{Bibliography, Content, Link, LinkDate, Linkable, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

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
	fn parse_content(
		content: &str,
		sack: &Sack,
		path: &Utf8Path,
		library: Option<&Library>,
	) -> (String, Outline, Bibliography) {
		let parsed = content
			.split("\n-----\n")
			.map(|chunk| {
				chunk
					.split("\n---\n")
					.map(|slide| crate::text::md::parse(&slide, sack, path, library).0)
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
		(parsed, Outline(vec![]), Bibliography(None))
	}

	fn as_html(&self, parsed: &str, sack: &Sack, _: Outline, _: Bibliography) -> String {
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

pub fn show(fm: &Slideshow, sack: &Sack, slides: &str) -> String {
	crate::html::bare(
		sack,
		maud!(
			div .reveal {
				div .slides {
					(Raw(slides))
				}
			}

			script type="module" {
				(Raw("import 'reveal'; import 'search';"))
			}

			style { (Raw(CSS)) }
		),
		fm.title.clone(),
	)
	.render()
	.into()
}
