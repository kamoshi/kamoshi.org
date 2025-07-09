use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use chrono::Datelike;
use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Renderable, html_elements, maud_move};

use crate::{Context, LinkDate, html::page};

/// Styles relevant to this fragment
const STYLES: &[&str] = &[
    "styles/styles.scss",
    "styles/layouts/list.scss",
    "styles/layouts/tags.scss",
];

pub fn group(links: &[LinkDate]) -> Vec<(i32, Vec<&LinkDate>)> {
    let mut groups = HashMap::<_, Vec<_>>::new();

    for page in links {
        groups.entry(page.date.year()).or_default().push(page);
    }

    let mut groups: Vec<_> = groups
        .into_iter()
        .map(|(k, mut v)| {
            v.sort_by(|a, b| b.date.cmp(&a.date));
            (k, v)
        })
        .collect();

    groups.sort_by(|a, b| b.0.cmp(&a.0));
    groups
}

pub fn render_tag<'ctx>(
    ctx: &'ctx Context,
    links: &[(i32, Vec<&'ctx LinkDate>)],
    title: String,
) -> TaskResult<impl Renderable> {
    let heading = title.clone();
    let list = maud_move!(
        main .page-list-main {
            article .page-list {
                header .directory-header .markdown {
                    h1 { (heading) }
                }

                @for (year, group) in links {
                    (section(*year, group))
                }
            }
        }
    );

    page(ctx, list, title, STYLES, Cow::default())
}

fn section(year: i32, group: &[&LinkDate]) -> impl Renderable {
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

fn link(data: &LinkDate) -> impl Renderable {
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

pub fn tag_cloud<'ctx>(
    ctx: &'ctx Context,
    tag_map: &'ctx BTreeMap<String, Vec<LinkDate>>,
    title: &'ctx str,
) -> TaskResult<impl Renderable> {
    let sorted = {
        let mut vec: Vec<_> = tag_map.iter().collect();
        vec.sort_by_key(|(tag, _)| tag.to_lowercase());
        vec
    };

    let main = maud_move! {
        main {
            article {
                header {
                    h1 { (title) }
                }
                ul {
                    @for (tag, entries) in sorted {
                        @let count = entries.len();

                        li {
                            a href=(format!("/tags/{tag}/")) title=(format!("{count} posts")) {
                                (tag)
                            }
                        }
                    }
                }
            }
        }
    };

    page(ctx, main, "Tag index".into(), STYLES, Cow::default())
}
