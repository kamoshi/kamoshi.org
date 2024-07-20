use hauchiwa::{LinkDate, Sack};
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::html::page;

pub fn list<'s, 'g, 'html>(
	sack: &'s Sack,
	groups: &'g [(i32, Vec<LinkDate>)],
	title: String,
) -> impl Renderable + 'html
where
	's: 'html,
	'g: 'html,
{
	let heading = title.clone();
	let list = maud_move!(
		main .page-list-main {
			article .page-list {
				header .markdown {
					h1 { (heading) }
				}

				@for (year, group) in groups {
					(section(*year, group))
				}
			}
		}
	);

	page(sack, list, title)
}

fn section(year: i32, group: &[LinkDate]) -> impl Renderable + '_ {
	maud_move!(
		section .page-list-year {
			header .page-list-year__header {
				h2 { (year) }
			}
			@for item in group.iter() {
				(link(item))
			}
		}
	)
}

fn link(data: &LinkDate) -> impl Renderable + '_ {
	let time = data.date.format("%m/%d");
	maud_move!(
		a .page-item href=(data.link.path.as_str()) {
			div .page-item__header {
				h3 {
					(&data.link.name)
				}
				time datetime=(data.date.to_rfc3339()) {
					(time.to_string())
				}
			}
			@if let Some(ref desc) = data.link.desc {
				div .page-item__desc {
					(desc)
				}
			}
		}
	)
}
