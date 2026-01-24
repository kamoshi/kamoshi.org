use std::collections::{BTreeMap, HashMap};

use camino::Utf8PathBuf;
use chrono::Datelike;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Document, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::prelude::*;

use crate::{Context, Global, Link, LinkDate, model::Post};

use super::make_page;

pub fn build_tags(
    config: &mut Blueprint<Global>,
    posts: Handle<Assets<Document<Post>>>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    Ok(task!(config, |ctx, posts, styles| {
        use std::collections::BTreeMap;

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/list.scss")?,
            styles.get("styles/layouts/tags.scss")?,
        ];

        let posts = posts
            .values()
            .filter(|item| !item.metadata.draft)
            .collect::<Vec<_>>();

        let mut tag_map: BTreeMap<String, Vec<LinkDate>> = BTreeMap::new();

        for post in &posts {
            for tag in &post.metadata.tags {
                tag_map.entry(tag.clone()).or_default().push(LinkDate {
                    link: Link {
                        path: Utf8PathBuf::from(&post.href),
                        name: post.metadata.title.clone(),
                        desc: post.metadata.desc.clone(),
                    },
                    date: post.metadata.date,
                });
            }
        }

        let mut pages = Vec::new();

        // Render individual tag pages
        for (tag, links) in &tag_map {
            let path = format!("tags/{tag}/index.html");

            let data = group(links);
            let html = render_tag(ctx, &data, tag.to_owned(), styles)?
                .render()
                .into_inner();

            pages.push(Output::html(path, html));

            // Render global tag index
            // let index = crate::html::tags::tag_cloud(&ctx, &tag_map, "Tag index")?;
            // pages.push(Page::text("tags/index.html".into(), index.render().into()));
        }

        Ok(pages)
    }))
}

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
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable, RuntimeError> {
    let heading = title.clone();
    let list = maud!(
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

    make_page(ctx, list, title, styles, &[])
}

fn section(year: i32, group: &[&LinkDate]) -> impl Renderable {
    maud!(
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
    maud!(
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
    styles: &'ctx [&Stylesheet],
) -> Result<impl Renderable, RuntimeError> {
    let sorted = {
        let mut vec: Vec<_> = tag_map.iter().collect();
        vec.sort_by_key(|(tag, _)| tag.to_lowercase());
        vec
    };

    let main = maud! {
        main {
            article {
                header {
                    h1 { (title) }
                }
                ul {
                    @for (tag, entries) in &sorted {
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

    make_page(ctx, main, "Tag index".into(), styles, &[])
}
