// mod hook;
mod markdown;
mod model;
mod plugin;
// mod rss;
mod ts;
mod typst;

use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::Site;
use hauchiwa::error::RuntimeError;
use hauchiwa::executor::run_once;
use hauchiwa::{SiteConfig, page::Page};
// use hauchiwa::loader::{self, Script, Svelte};
// use hauchiwa::{Hook, Page, RuntimeError, Website};
use hayagriva::Library;
use hypertext::{Raw, Renderable};
// use model::Slideshow;

use crate::plugin::about::build_about;
use crate::plugin::home::build_home;
use crate::plugin::slides::build_slides;
use crate::plugin::twtxt::build_twtxt;
use crate::plugin::{make_fullscreen, make_page};

/// Base path for content files
const CONTENT: &str = "content";
const BASE_URL: &str = "https://kamoshi.org/";

#[derive(Parser, Debug, Clone)]
struct Args {
    #[clap(value_enum, index = 1, default_value = "build")]
    mode: Mode,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Mode {
    Build,
    Watch,
}

pub struct Bibliography(pub Option<Vec<String>>);

#[derive(Debug, Clone)]
struct Global {
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}

impl Global {
    fn new() -> Self {
        let time = chrono::Utc::now();
        Self {
            year: time.year(),
            date: time.format("%Y/%m/%d %H:%M").to_string(),
            link: "https://github.com/kamoshi/kamoshi.org".into(),
            hash: String::from_utf8(
                Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .output()
                    .expect("Couldn't load git revision")
                    .stdout,
            )
            .expect("Invalid UTF8")
            .trim()
            .into(),
        }
    }
}

// convenient wrapper for `Context`
type Context = hauchiwa::Globals<Global>;

#[derive(Debug, Clone)]
struct Link {
    pub path: Utf8PathBuf,
    pub name: String,
    pub desc: Option<String>,
}

#[derive(Debug, Clone)]
struct LinkDate {
    pub link: Link,
    pub date: DateTime<Utc>,
}

struct Bibtex {
    path: Utf8PathBuf,
    data: Library,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), RuntimeError> {
    let args = Args::parse();

    let mut site = SiteConfig::new();

    let styles =
        hauchiwa::loader::build_styles(&mut site, "styles/**/[!_]*.scss", "styles/**/*.scss");

    let scripts =
        hauchiwa::loader::build_scripts(&mut site, "scripts/**/main.ts", "scripts/**/*.ts");

    let svelte =
        hauchiwa::loader::build_svelte(&mut site, "scripts/**/App.svelte", "scripts/**/*.svelte");

    build_home(&mut site, styles, svelte);
    build_about(&mut site, styles);
    build_twtxt(&mut site, styles);
    build_slides(&mut site, styles, scripts);

    site.add_task(
        (styles, scripts, svelte),
        |ctx, (styles, scripts, svelte)| {
            let mut pages = vec![];

            {
                let html = Raw(r#"<div id="map" style="height: 100%; width: 100%"></div>"#);

                let styles = &[
                    styles.get("styles/styles.scss").unwrap(),
                    styles.get("styles/photos/leaflet.scss").unwrap(),
                    styles.get("styles/layouts/map.scss").unwrap(),
                ];

                let scripts = &[scripts.get("scripts/photos/main.ts").unwrap()];

                let html = make_fullscreen(&ctx, html, "Map".into(), styles, scripts)
                    .unwrap()
                    .render();

                pages.push(Page {
                    url: "map".into(),
                    content: html.into_inner(),
                });
            }

            {
                let styles = &[
                    styles.get("styles/styles.scss").unwrap(),
                    styles.get("styles/layouts/search.scss").unwrap(),
                ];

                let component = svelte.get("scripts/search/App.svelte").unwrap();
                let scripts = &[&component.init];

                let html = (component.html)(&()).unwrap();
                let html = Raw(format!(r#"<main>{html}</main>"#));
                let html = make_page(&ctx, html, "Search".into(), styles, scripts)
                    .unwrap()
                    .render();

                pages.push(Page {
                    url: "search".into(),
                    content: html.into_inner(),
                });
            }

            pages
        },
    );

    let mut site = Site::new(site);
    let globals = hauchiwa::Globals {
        generator: "hauchiwa",
        mode: hauchiwa::Mode::Build,
        port: None,
        data: Global::new(),
    };
    let (_, pages) = run_once(&mut site, &globals);

    for page in pages {
        println!("Page: {} ({} bytes)", page.url, page.content.len());
    }

    // let mut website = Website::config()
    //     .load_git(".")?
    //     // .add_plugins([
    //     //     plugin::home::PLUGIN,
    //     //     plugin::about::PLUGIN,
    //     //     plugin::posts::PLUGIN,
    //     //     plugin::slides::PLUGIN,
    //     //     plugin::projects::PLUGIN,
    //     //     plugin::wiki::PLUGIN,
    //     //     plugin::twtxt::PLUGIN,
    //     //     plugin::tags::PLUGIN,
    //     // ])
    //     .add_loaders([
    //         // .bib -> Bibtex
    //         loader::glob_assets(CONTENT, "**/*.bib", |rt, data| {
    //             let path = rt.store(&data, "bib")?;
    //             let text = String::from_utf8_lossy(&data);
    //             let data = hayagriva::io::from_biblatex_str(&text).unwrap();

    //             Ok(Bibtex { path, data })
    //         }),
    //         // images
    //         loader::glob_images(CONTENT, "**/*.jpg"),
    //         loader::glob_images(CONTENT, "**/*.png"),
    //         loader::glob_images(CONTENT, "**/*.gif"),
    //         // svelte components
    //         loader::glob_svelte::<()>("scripts", "*/App.svelte"),
    //         // loader::glob_svelte::<Mermaid>("scripts", "mermaid/Mermaid.svelte"),
    //         // stylesheets
    //         // scripts
    //         // github
    //         loader::async_asset("hauchiwa", async |_| {
    //             const URL: &str =
    //                 "https://raw.githubusercontent.com/kamoshi/hauchiwa/refs/heads/main/README.md";

    //             Ok(reqwest::get(URL).await?.text().await?)
    //         }),
    //     ])
    //     .add_hook(Hook::post_build(crate::hook::build_pagefind))
    //     .add_hook(Hook::post_build(crate::hook::build_sitemap))
    //     .finish();

    // match args.mode {
    //     Mode::Build => website.build(Global::new())?,
    //     Mode::Watch => website.watch(Global::new())?,
    // };

    Ok(())
}
