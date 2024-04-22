use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use crate::html::page;


#[derive(Debug, Clone)]
pub struct Link {
    pub path: Utf8PathBuf,
    pub name: String,
    pub desc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LinkDate {
    pub link: Link,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum Linkable {
    Link(Link),
    Date(LinkDate),
}


pub fn list<'data, 'list>(
    title: &'data str,
    groups: &'data [(i32, Vec<LinkDate>)]
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
