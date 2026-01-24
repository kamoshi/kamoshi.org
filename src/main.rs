mod datalog;
mod markdown;
mod md;
mod model;
mod plugin;
mod rss;
mod ts;
mod typst;

use std::collections::HashSet;
use std::fs;
use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::error::RuntimeError;
use hauchiwa::loader::image::ImageFormat;
use hauchiwa::page::Output;
use hauchiwa::{TaskContext, Website, task};
use hayagriva::Library;
use hypertext::{Raw, Renderable};

use crate::plugin::about::build_about;
use crate::plugin::home::build_home;
use crate::plugin::posts::build_posts;
use crate::plugin::projects::build_projects;
use crate::plugin::slides::build_slides;
use crate::plugin::tags::build_tags;
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
    pub repo: hauchiwa::git::GitRepo,
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}

impl Global {
    fn new() -> Self {
        use hauchiwa::git;

        let time = chrono::Utc::now();

        Self {
            repo: git::map(git::Options::new("main")).unwrap(),
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
type Context<'a> = TaskContext<'a, Global>;

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

    let mut config = Website::<Global>::design();

    let images = config
        .load_images()
        .format(ImageFormat::WebP)
        .source("content/**/*.jpg")
        .source("content/**/*.png")
        .source("content/**/*.gif")
        .register()?;

    let styles = config.load_css("styles/**/[!_]*.scss", "styles/**/*.scss")?;
    let scripts = config.load_js("scripts/**/main.ts", "scripts/**/*.ts")?;
    let svelte = config.load_svelte("scripts/**/App.svelte", "scripts/**/*.svelte")?;

    let bibtex = config.load("**/*.bib", |_, store, input| {
        let data = input.read()?;
        let path = store.save(&data, "bib")?;
        let text = String::from_utf8_lossy(&data);
        let data = hayagriva::io::from_biblatex_str(&text).unwrap();

        Ok(Bibtex { path, data })
    })?;

    let home = build_home(&mut config, images, styles, svelte)?;
    let about = build_about(&mut config, images, styles)?;
    let _ = build_twtxt(&mut config, styles)?;
    let (posts_data, posts) = build_posts(&mut config, images, styles, scripts, bibtex)?;
    let slides = build_slides(&mut config, images, styles, scripts)?;
    let _ = build_projects(&mut config, styles)?;
    let _ = build_tags(&mut config, posts_data, styles)?;

    // digital garden
    let _ = crate::plugin::wiki::build(&mut config, images, styles)?;

    let other = task!(config, |ctx, styles, scripts, svelte| {
        let mut pages = vec![];

        {
            let html = Raw::dangerously_create(
                r#"<div id="map" style="height: 100%; width: 100%"></div>"#,
            );

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/photos/leaflet.scss")?,
                styles.get("styles/layouts/map.scss")?,
            ];

            let scripts = &[scripts.get("scripts/photos/main.ts")?];

            let html = make_fullscreen(ctx, html, "Map".into(), styles, scripts)?
                .render()
                .into_inner();

            pages.push(Output::html("map", html));
        }

        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/search.scss")?,
            ];

            let component = svelte.get("scripts/search/App.svelte")?;
            let scripts = &[&component.hydration];

            let html = (component.prerender)(&())?;
            let html = Raw::dangerously_create(format!(r#"<main>{html}</main>"#));
            let html = make_page(ctx, html, "Search".into(), styles, scripts)?
                .render()
                .into_inner();

            pages.push(Output::html("search", html));
        }

        Ok(pages)
    });

    task!(config, |_, home, about, posts, slides, other| {
        use pagefind::api::PagefindIndex;
        use pagefind::options::PagefindServiceConfig;
        use tokio::runtime::Builder;

        let run = async move |pages: &[&Output]| -> Result<(), RuntimeError> {
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
            .flat_map(|source| source.iter())
            .collect::<Vec<_>>();

        Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(run(&pages))?;

        Ok(())
    });

    task!(config, |_, home, about, posts, slides, other| {
        use sitemap_rs::{
            url::{ChangeFrequency, Url},
            url_set::UrlSet,
        };

        let pages = [&home, &about, &posts, &slides, &other]
            .into_iter()
            .flat_map(|source| source.iter())
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

        Ok(Output::file("sitemap.xml", String::from_utf8(buf).unwrap()))
    });

    let mut website = config.finish();

    match args.mode {
        Mode::Build => {
            website
                .build(Global::new())?
                .render_waterfall_to_file(&website, "waterfall.svg")?;
        }
        Mode::Watch => {
            website.watch(Global::new())?;
        }
    };

    Ok(())
}
