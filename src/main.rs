mod markdown;
mod model;
mod plugin;
// mod rss;
mod ts;
mod typst;

use std::collections::HashSet;
use std::fs;
use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::error::RuntimeError;
use hauchiwa::executor::run_once_parallel;
use hauchiwa::loader::{Runtime, glob_assets, glob_images};
use hauchiwa::page::save_pages_to_dist;
use hauchiwa::{Site, task};
use hauchiwa::{SiteConfig, page::Page};
use hayagriva::Library;
use hypertext::{Raw, Renderable};

use crate::plugin::about::build_about;
use crate::plugin::home::build_home;
use crate::plugin::posts::build_posts;
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

#[derive(Debug, Clone)]
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

    let _ = fs::remove_dir_all("./dist");

    let mut site = SiteConfig::new();

    let styles =
        hauchiwa::loader::build_styles(&mut site, "styles/**/[!_]*.scss", "styles/**/*.scss");

    let scripts =
        hauchiwa::loader::build_scripts(&mut site, "scripts/**/main.ts", "scripts/**/*.ts");

    let svelte =
        hauchiwa::loader::build_svelte(&mut site, "scripts/**/App.svelte", "scripts/**/*.svelte");

    // images
    let images = glob_images(&mut site, "**/*.jpg");
    // glob_images(&mut site, "**/*.png");
    // glob_images(&mut site, "**/*.gif");

    let bibtex = glob_assets(&mut site, "**/*.bib", |_, file| {
        let rt = Runtime;
        let path = rt.store(&file.metadata, "bib")?;
        let text = String::from_utf8_lossy(&file.metadata);
        let data = hayagriva::io::from_biblatex_str(&text).unwrap();

        Ok(Bibtex { path, data })
    });

    let home = build_home(&mut site, styles, svelte);
    let about = build_about(&mut site, styles);
    let twtxt = build_twtxt(&mut site, styles);
    let posts = build_posts(&mut site, styles, scripts);
    let slides = build_slides(&mut site, styles, scripts);

    let other = task!(site, |ctx, styles, scripts, svelte| {
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

            pages.push(Page::html("map", html));
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

            pages.push(Page::html("search", html));
        }

        pages
    });

    task!(site, |_, home, about, posts, slides, other| {
        use pagefind::api::PagefindIndex;
        use pagefind::options::PagefindServiceConfig;
        use tokio::runtime::Builder;

        let run = async move |pages: &[&Page]| -> Result<(), RuntimeError> {
            let config = PagefindServiceConfig::builder().build();
            let mut index = PagefindIndex::new(Some(config))?;

            for page in pages {
                if let Some("html") = page.url.extension() {
                    index
                        .add_html_file(Some(page.url.to_string()), None, page.content.to_string())
                        .await?;
                }
            }

            let _ = index.write_files(Some("dist/pagefind".into())).await?;

            Ok(())
        };

        let pages = [&home, &about, &posts, &slides, &other]
            .into_iter()
            .flat_map(|source| source.into_iter())
            .collect::<Vec<_>>();

        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run(&pages))
            .unwrap();
    });

    task!(site, |_, home, about, posts, slides, other| {
        use sitemap_rs::{
            url::{ChangeFrequency, Url},
            url_set::UrlSet,
        };

        let pages = [&home, &about, &posts, &slides, &other]
            .into_iter()
            .flat_map(|source| source.into_iter())
            .collect::<Vec<_>>();

        let urls = pages
            .iter()
            .map(|page| &page.url)
            .collect::<HashSet<_>>()
            .iter()
            .map(|path| {
                Url::builder(Utf8Path::new("/").join(path).parent().unwrap().to_string())
                    .change_frequency(ChangeFrequency::Monthly)
                    .priority(0.8)
                    .build()
                    .expect("failed a <url> validation")
            })
            .collect::<Vec<_>>();

        let urls = UrlSet::new(urls).expect("failed a <urlset> validation");
        let mut buf = Vec::<u8>::new();
        urls.write(&mut buf).expect("failed to write XML");

        Page::file("sitemap.xml", String::from_utf8(buf).unwrap())
    });

    let mut site = Site::new(site);
    let globals = hauchiwa::Globals {
        generator: "hauchiwa",
        mode: hauchiwa::Mode::Build,
        port: None,
        data: Global::new(),
    };
    let (_, pages) = run_once_parallel(&mut site, &globals);

    save_pages_to_dist(&pages);

    // let mut website = Website::config()
    //     .load_git(".")?
    //     // .add_plugins([
    //     //     plugin::projects::PLUGIN,
    //     //     plugin::wiki::PLUGIN,
    //     //     plugin::tags::PLUGIN,
    //     // ])
    //     .add_loaders([
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
