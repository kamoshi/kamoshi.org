mod head;
mod home;
mod list;
mod misc;
pub mod post;
pub mod project;
pub mod slideshow;
pub mod wiki;

use std::collections::HashMap;

use camino::Utf8Path;
use chrono::Datelike;
use hauchiwa::TaskResult;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud, maud_move};

pub(crate) use home::home;

use crate::{Context, LinkDate};

fn navbar() -> impl Renderable {
    static ITEMS: &[(&str, &str)] = &[
        ("Posts", "/posts/"),
        ("Slides", "/slides/"),
        ("Projects", "/projects/"),
        ("Wiki", "/wiki/"),
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
                            (Raw(include_str!("logotype.svg")))
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

pub fn footer(sack: &Context) -> impl Renderable {
    let globals = sack.get_globals();

    let copy = format!("Copyright &copy; {} Maciej Jur", &globals.data.year);
    let mail = "maciej@kamoshi.org";
    let href = format!("mailto:{}", mail);
    let link = Utf8Path::new(&globals.data.link)
        .join("src/commit")
        .join(&globals.data.hash);

    // let link = match sack.get_file() {
    // 	Some(path) => link.join(path),
    // 	None => link,
    // };

    maud_move!(
        footer .footer {
            div .left {
                div {
                    (Raw(copy))
                }
                a href=(href)  {
                    (mail)
                }
            }
            div .repo {
                a href=(link.as_str()) {
                    (&sack.get_globals().data.hash)
                }
                div {
                    (&sack.get_globals().data.date)
                }
            }
            a .right.footer__cc-wrap rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
                img .footer__cc-stamp alt="Creative Commons License" width="88" height="31" src="/static/svg/by.svg";
            }
        }
    )
}

fn bare<'s, 'p, 'html>(
    sack: &'s Context,
    main: impl Renderable + 'p,
    title: String,
    stylesheets: &'s [&str],
    js: Option<&'s [String]>,
) -> TaskResult<impl Renderable>
where
    's: 'html,
    'p: 'html,
{
    let head = head::render_head(sack, title, stylesheets, js)?;

    Ok(maud_move!(
        (Raw("<!DOCTYPE html>"))
        html lang="en" {
            (head)

            body {
                (main)
            }
        }
    ))
}

fn full<'s, 'p, 'html>(
    sack: &'s Context,
    main: impl Renderable + 'p,
    title: String,
    js: Option<&'s [String]>,
) -> TaskResult<impl Renderable + 'html>
where
    's: 'html,
    'p: 'html,
{
    let main = maud_move!((navbar())(main));

    bare(
        sack,
        main,
        title,
        &[
            "styles/styles.scss",
            "styles/photos/leaflet.scss",
            "styles/layouts/map.scss",
        ],
        js,
    )
}

fn page<'s, 'p, 'html>(
    sack: &'s Context,
    main: impl Renderable + 'p,
    title: String,
    stylesheets: &'s [&str],
    js: Option<&'s [String]>,
) -> TaskResult<impl Renderable>
where
    's: 'html,
    'p: 'html,
{
    let main = maud_move!((navbar())(main)(footer(sack)));

    bare(sack, main, title, stylesheets, js)
}

pub(crate) fn to_list(
    sack: &Context,
    list: Vec<LinkDate>,
    title: String,
    rss: &'static str,
) -> String {
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

    list::list(sack, &groups, title, rss)
        .unwrap()
        .render()
        .into()
}

pub(crate) fn map<'s, 'html>(
    sack: &'s Context,
    js: Option<&'s [String]>,
) -> TaskResult<impl Renderable>
where
    's: 'html,
{
    full(
        sack,
        maud!(
            div #map style="height: 100%; width: 100%" {}

            script type="module" {
                (Raw("import 'photos';"))
            }
        ),
        String::from("Map"),
        js,
    )
}

pub(crate) fn search(sack: &Context) -> String {
    page(
        sack,
        maud!(
            main #app {}
        ),
        String::from("Search"),
        &["styles/styles.scss", "styles/layouts/search.scss"],
        Some(&["search".into()]),
    )
    .unwrap()
    .render()
    .into()
}

// pub fn as_html(
//     meta: &Post,
//     parsed: &str,
//     sack: &MySack,
//     outline: Outline,
//     bibliography: Bibliography,
// ) -> String {
//     flox(meta, parsed, sack, outline, bibliography)
// }

// pub(crate) fn flox(
//     meta: &Post,
//     parsed: &str,
//     sack: &MySack,
//     outline: Outline,
//     bibliography: Bibliography,
// ) -> String {
//     page(
//         sack,
//         maud_move!(
//             main {
//                 div .flox-playground {
//                     div .cell {
//                         header {
//                             h2 { "Flox" }
//                         }
//                         div .editor {
//                             div #editor {}
//                             button #run .run { "Run!" }
//                         }
//                     }
//                     div .cell {
//                         h2 { "Output" }
//                         pre #output {}
//                     }
//                 }
//                 (article(meta, parsed, sack, outline, bibliography))
//             }
//         ),
//         String::from("Flox"),
//         &["styles/styles.scss", "styles/layouts/page.scss"],
//         Some(&["editor".into()]),
//     )
//     .unwrap()
//     .render()
//     .into()
// }
