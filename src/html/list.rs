use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Renderable, html_elements, maud_move};

use crate::{LinkDate, MySack, html::page};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/list.scss"];

const ICON_RSS: &str = include_str!("rss.svg");

pub fn list<'s, 'g, 'html>(
    sack: &'s MySack,
    groups: &'g [(i32, Vec<LinkDate>)],
    title: String,
    rss: &'static str,
) -> TaskResult<impl Renderable>
where
    's: 'html,
    'g: 'html,
{
    let heading = title.clone();
    let list = maud_move!(
        main .page-list-main {
            article .page-list {
                header .directory-header .markdown {
                    h1 { (heading) }
                    a href=(rss) title="RSS feed" {
                       (hypertext::Raw(ICON_RSS))
                    }
                }

                @for (year, group) in groups {
                    (section(*year, group))
                }
            }
        }
    );

    page(sack, list, title, STYLES, None)
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
