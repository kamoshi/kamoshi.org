pub mod about;
pub mod home;
pub mod posts;
pub mod projects;
pub mod slides;
pub mod tags;
pub mod twtxt;
pub mod wiki;

use std::collections::HashMap;

use camino::Utf8Path;
use chrono::Datelike as _;
use hauchiwa::{
    error::RuntimeError,
    loader::{Script, Stylesheet, TemplateEnv},
};
use minijinja::Value;

use crate::props::{
    PropsFooter, PropsHead, PropsList, PropsListGroup, PropsListItem, PropsNavItem, PropsNavbar,
};
use crate::{Context, LinkDate};

const LOGOTYPE_SVG: &str = include_str!("../assets/logotype.svg");
const ICON_RSS: &str = include_str!("../assets/rss.svg");

const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("綴", "Posts", "/posts/"),
    ("映", "Slides", "/slides/"),
    ("創", "Projects", "/projects/"),
    ("葉", "Garden", "/wiki/"),
    ("想", "Journal", "/thoughts/"),
    ("跡", "Map", "/map/"),
    ("己", "About", "/about/"),
    ("索", "Search", "/search/"),
];

pub(crate) fn make_props_navbar() -> PropsNavbar {
    PropsNavbar {
        logotype_svg: Value::from_safe_string(LOGOTYPE_SVG.to_string()),
        items: NAV_ITEMS
            .iter()
            .map(|&(stamp, name, url)| PropsNavItem { stamp, name, url })
            .collect(),
    }
}

pub(crate) fn make_props_footer(ctx: &Context) -> PropsFooter {
    let repo_link = Utf8Path::new(&ctx.env.data.link).join(&ctx.env.data.hash);
    PropsFooter {
        year: ctx.env.data.year,
        repo_link: repo_link.to_string(),
        hash_short: ctx.env.data.hash[0..7].to_string(),
        date: ctx.env.data.date.clone(),
    }
}

pub(crate) fn make_props_head(
    ctx: &Context,
    title: String,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<PropsHead, RuntimeError> {
    Ok(PropsHead {
        title,
        generator: ctx.env.generator,
        importmap: Value::from_safe_string(ctx.importmap.to_json()?),
        styles: styles.iter().map(|s| s.path.to_string()).collect(),
        scripts: scripts.iter().map(|s| s.path.to_string()).collect(),
        refresh_script: ctx.env.get_refresh_script().map(Value::from_safe_string),
    })
}

pub(crate) fn to_list(
    ctx: &Context,
    templates: &TemplateEnv,
    list: Vec<LinkDate>,
    title: String,
    rss: &'static str,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let mut groups = HashMap::<i32, Vec<_>>::new();

    for page in list {
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

    let props = PropsList {
        head: make_props_head(ctx, title.clone(), styles, &[])?,
        navbar: make_props_navbar(),
        footer: make_props_footer(ctx),
        title,
        rss,
        icon_rss: Value::from_safe_string(ICON_RSS.to_string()),
        groups: groups
            .into_iter()
            .map(|(year, items)| PropsListGroup {
                year,
                items: items
                    .into_iter()
                    .map(|item| PropsListItem {
                        path: item.link.path.to_string(),
                        name: item.link.name,
                        desc: item.link.desc,
                        date: item.date.format("%m/%d").to_string(),
                        date_iso: item.date.to_rfc3339(),
                    })
                    .collect(),
            })
            .collect(),
    };

    let tmpl = templates.get_template("list.jinja")?;
    Ok(tmpl.render(&props)?)
}
