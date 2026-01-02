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
    loader::{Script, Stylesheet},
};
// use hauchiwa::{RuntimeError, loader::Style};
use hypertext::{Raw, prelude::*};

use crate::{Context, LinkDate};

// use crate::{Context, LinkDate};

fn make_head(
    ctx: &Context,
    title: String,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<impl Renderable, RuntimeError> {
    let title = format!("{title} | kamoshi.org");

    let importmap = ctx.importmap.to_json()?;

    let html = maud!(
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        meta name="generator" content=(ctx.env.generator);

        title { (title) }

        link rel="sitemap" href="/sitemap.xml";

        link rel="preconnect" href="https://rsms.me/";
        link rel="stylesheet" href="https://rsms.me/inter/inter.css";

        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";

        script type="importmap" { (Raw::dangerously_create(&importmap)) }

        @for style in styles {
            link rel="stylesheet" href=(style.path.as_str());
        }

        @for script in scripts {
            script type="module" src=(script.path.as_str()) {}
        }

        @if let Some(reload_script) = ctx.env.get_refresh_script() {
            script { (Raw::dangerously_create(reload_script)) }
        }
    );

    Ok(html)
}

fn make_navbar() -> impl Renderable {
    const ITEMS: &[(&str, &str)] = &[
        ("Posts", "/posts/"),
        ("Slides", "/slides/"),
        ("Projects", "/projects/"),
        ("Wiki", "/wiki/"),
        ("Thoughts", "/thoughts/"),
        ("Map", "/map/"),
        ("About", "/about/"),
        ("Search", "/search/"),
    ];

    maud!(
        nav .p-nav {
            input #p-nav-toggle type="checkbox" hidden;

            div .p-nav__bar {
                a .p-nav__logo href="/" {
                    img .p-nav__logo-icon height="48px" width="51px" src="/static/svg/aya.svg" alt="";
                    div .p-nav__logo-text {
                        div .p-nav__logo-main {
                            (Raw::dangerously_create(include_str!("../assets/logotype.svg")))
                        }
                        div #p-nav-splash .p-nav__logo-sub {
                          "夢現の遥か彼方"
                        }
                    }
                }

                label .p-nav__burger for="p-nav-toggle" tabindex="0" {
                    span .p-nav__burger-icon {}
                }
            }

            menu .p-nav__menu {
                @for (name, url) in ITEMS {
                    li .p-nav__menu-item {
                        a .p-nav__menu-link href=(*url) {
                            (*name)
                        }
                    }
                }
            }
        }
    )
}

pub fn make_footer(ctx: &Context) -> impl Renderable {
    let copy = format!("Copyright &copy; {} Maciej Jur", &ctx.env.data.year);
    let mail = "maciej@kamoshi.org";
    let href = format!("mailto:{mail}");
    let link = Utf8Path::new(&ctx.env.data.link)
        .join("tree")
        .join(&ctx.env.data.hash);

    maud!(
        footer .footer {
            div .left {
                div {
                    (Raw::dangerously_create(&copy))
                }
                a href=(href)  {
                    (mail)
                }
            }
            div .repo {
                a href=(link.as_str()) {
                    (&ctx.env.data.hash)
                }
                div {
                    (&ctx.env.data.date)
                }
            }
            a .right.footer__cc-wrap rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
                img .footer__cc-stamp alt="Creative Commons License" width="88" height="31" src="/static/svg/by.svg";
            }
        }
    )
}

pub fn make_bare<'ctx>(
    ctx: &'ctx Context,
    main: impl Renderable + 'ctx,
    title: String,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<impl Renderable, RuntimeError> {
    let head = make_head(ctx, title, styles, scripts)?;

    Ok(maud!(
        !DOCTYPE
        html lang="en" {
            (head)

            body {
                (main)
            }
        }
    ))
}

pub fn make_fullscreen<'ctx>(
    ctx: &'ctx Context,
    main: impl Renderable + 'ctx,
    title: String,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<impl Renderable, RuntimeError> {
    let main = maud!(
        // navbar
        (make_navbar())
        // main
        (main)
    );

    make_bare(ctx, main, title, styles, scripts)
}

pub fn make_page<'ctx>(
    sack: &'ctx Context,
    main: impl Renderable + 'ctx,
    title: String,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<impl Renderable, RuntimeError> {
    let main = maud!(
        // navbar
        (make_navbar())
        // main
        (main)
        // footer
        (make_footer(sack))
    );

    make_bare(sack, main, title, styles, scripts)
}

const ICON_RSS: &str = include_str!("../assets/rss.svg");

pub(crate) fn to_list(
    sack: &Context,
    list: Vec<LinkDate>,
    title: String,
    rss: &'static str,
    styles: &[&Stylesheet],
) -> Result<impl Renderable, RuntimeError> {
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

    let heading = title.clone();
    let list = maud!(
        main .page-list-main {
            article .page-list {
                header .directory-header .markdown {
                    h1 { (heading) }
                    a href=(rss) title="RSS feed" {
                       (Raw::dangerously_create(ICON_RSS))
                    }
                }

                @for (year, group) in &groups {
                    (section(*year, group))
                }
            }
        }
    );

    make_page(sack, list, title, styles, &[])
}

fn section(year: i32, group: &[LinkDate]) -> impl Renderable + '_ {
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

fn link(data: &LinkDate) -> impl Renderable + '_ {
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

pub fn render_bibliography(bib: &[String], library_path: Option<&Utf8Path>) -> impl Renderable {
    maud!(
        section .bibliography {
            header {
                h2 {
                    "Bibliography"
                }
                @if let Some(path) = library_path {
                    a.icon-btn href=(path.as_str()) download="bibliography.bib" title="Download BibTeX" {
                        img src="/static/svg/lucide/file-down.svg" alt="Download";
                    }
                }
            }
            ol {
                @for item in bib {
                    li {
                        (Raw::dangerously_create(item))
                    }
                }
            }
        }
    )
}
