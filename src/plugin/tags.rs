use std::collections::{BTreeMap, HashMap};

use camino::Utf8PathBuf;
use chrono::Datelike;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Document, Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;

use crate::props::{PropsListGroup, PropsListItem, PropsTag, PropsTagCloud, PropsTagCloudEntry};
use crate::{Context, Global, Link, LinkDate, model::Post};

pub fn add_tags(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    posts: Many<Document<Post>>,
    styles: Many<Stylesheet>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let handle = config
        .task()
        .using((templates, posts, styles))
        .merge(|ctx, (templates, posts, styles)| {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/list.scss")?,
                styles.get("styles/layouts/tags.scss")?,
            ];

            let posts = posts
                .values()
                .filter(|item| !item.matter.draft)
                .collect::<Vec<_>>();

            let mut tag_map: BTreeMap<String, Vec<LinkDate>> = BTreeMap::new();

            for post in &posts {
                for tag in &post.matter.tags {
                    tag_map.entry(tag.clone()).or_default().push(LinkDate {
                        link: Link {
                            path: Utf8PathBuf::from(&post.meta.href),
                            name: post.matter.title.clone(),
                            desc: post.matter.desc.clone(),
                        },
                        date: post.matter.date,
                    });
                }
            }

            let mut pages = Vec::new();

            for (tag, links) in &tag_map {
                let path = format!("tags/{tag}/index.html");
                let html = render_tag(ctx, templates, &group(links), tag.to_owned(), styles)?;
                pages.push(Output::html(path, html));
            }

            Ok(pages)
        });

    Ok(handle)
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

pub fn render_tag(
    ctx: &Context,
    templates: &TemplateEnv,
    links: &[(i32, Vec<&LinkDate>)],
    title: String,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let props = PropsTag {
        head: super::make_props_head(ctx, title.clone(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        title,
        groups: links
            .iter()
            .map(|(year, items)| PropsListGroup {
                year: *year,
                items: items
                    .iter()
                    .map(|item| PropsListItem {
                        path: item.link.path.to_string(),
                        name: item.link.name.clone(),
                        desc: item.link.desc.clone(),
                        date: item.date.format("%m/%d").to_string(),
                        date_iso: item.date.to_rfc3339(),
                    })
                    .collect(),
            })
            .collect(),
    };

    let tmpl = templates.get_template("tag.jinja")?;
    Ok(tmpl.render(&props)?)
}

pub fn tag_cloud(
    ctx: &Context,
    templates: &TemplateEnv,
    tag_map: &BTreeMap<String, Vec<LinkDate>>,
    title: &str,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let mut entries: Vec<_> = tag_map.iter().collect();
    entries.sort_by_key(|(tag, _)| tag.to_lowercase());

    let props = PropsTagCloud {
        head: super::make_props_head(ctx, title.to_string(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        title: title.to_string(),
        entries: entries
            .iter()
            .map(|(tag, items)| PropsTagCloudEntry {
                tag: tag.to_string(),
                count: items.len(),
            })
            .collect(),
    };

    let tmpl = templates.get_template("tag_cloud.jinja")?;
    Ok(tmpl.render(&props)?)
}
