use chrono::{DateTime, Utc};
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use crate::html::page;


#[derive(Clone)]
pub struct LinkableData {
    pub path: String,
    pub name: String,
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
}


pub fn list<'data, 'list>(
    title: &'data str,
    groups: &'data [(i32, Vec<LinkableData>)]
) -> impl Renderable + 'list
    where
        'data: 'list
{
    let list = maud_move!(
        main .page-list-main {
            article .page-list {
                header .markdown {
                    h1 { (title) }
                }

                @for (year, group) in groups {
                    (section(*year, group))
                }
            }
        }
    );

    page(title, list)
}

fn section(year: i32, group: &[LinkableData]) -> impl Renderable + '_ {
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

fn link(data: &LinkableData) -> impl Renderable + '_ {
    let time = data.date.format("%m/%d");
    maud_move!(
        a .page-item href=(&data.path) {
            div .page-item__header {
                h3 {
                    (&data.name)
                }
                time datetime=(data.date.to_rfc3339()) {
                    (time.to_string())
                }
            }
            @if let Some(ref desc) = data.desc {
                div .page-item__desc {
                    (desc)
                }
            }
        }
    )
}
