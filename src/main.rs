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
use hauchiwa::loader::glob_assets;
use hauchiwa::{Site, task};
use hauchiwa::{SiteConfig, page::Page};
use hayagriva::Library;
use hypertext::{Raw, Renderable};

use crate::plugin::about::build_about;
use crate::plugin::home::build_home;
use crate::plugin::posts::build_posts;
use crate::plugin::projects::build_projects;
use crate::plugin::slides::build_slides;
use crate::plugin::tags::build_tags;
use crate::plugin::twtxt::build_twtxt;
use crate::plugin::wiki::build_wiki;
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
    pub repo: hauchiwa::gitmap::GitRepo,
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}

impl Global {
    fn new() -> Self {
        use hauchiwa::gitmap;

        let time = chrono::Utc::now();

        let git = gitmap::map(gitmap::Options::new("main")).unwrap();

        Self {
            repo: git,
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
type Context<'a> = hauchiwa::Context<'a, Global>;

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

    let styles = site.build_styles("styles/**/[!_]*.scss", "styles/**/*.scss")?;
    let scripts = site.build_scripts("scripts/**/main.ts", "scripts/**/*.ts")?;
    let svelte = site.build_svelte("scripts/**/App.svelte", "scripts/**/*.svelte")?;

    // images
    let images = site.glob_images(&["**/*.jpg", "**/*.png", "**/*.gif"])?;

    let bibtex = glob_assets(&mut site, "**/*.bib", |_, rt, file| {
        let path = rt.store(&file.metadata, "bib")?;
        let text = String::from_utf8_lossy(&file.metadata);
        let data = hayagriva::io::from_biblatex_str(&text).unwrap();

        Ok(Bibtex { path, data })
    })?;

    let home = build_home(&mut site, images, styles, svelte)?;
    let about = build_about(&mut site, images, styles)?;
    let _ = build_twtxt(&mut site, styles)?;
    let (posts_data, posts) = build_posts(&mut site, images, styles, scripts, bibtex)?;
    let slides = build_slides(&mut site, images, styles, scripts)?;
    let _ = build_wiki(&mut site, images, styles)?;
    let _ = build_projects(&mut site, styles)?;
    let _ = build_tags(&mut site, posts_data, styles)?;

    let other = task!(site, |ctx, styles, scripts, svelte| {
        let mut pages = vec![];

        {
            let html = Raw(r#"<div id="map" style="height: 100%; width: 100%"></div>"#);

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/photos/leaflet.scss")?,
                styles.get("styles/layouts/map.scss")?,
            ];

            let scripts = &[scripts.get("scripts/photos/main.ts")?];

            let html = make_fullscreen(ctx, html, "Map".into(), styles, scripts)?.render();

            pages.push(Page::html("map", html));
        }

        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/search.scss")?,
            ];

            let component = svelte.get("scripts/search/App.svelte")?;
            let scripts = &[&component.init];

            let html = (component.html)(&())?;
            let html = Raw(format!(r#"<main>{html}</main>"#));
            let html = make_page(ctx, html, "Search".into(), styles, scripts)?.render();

            pages.push(Page::html("search", html));
        }

        Ok(pages)
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
            .flat_map(|source| source.iter())
            .collect::<Vec<_>>();

        Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(run(&pages))?;

        Ok(())
    });

    task!(site, |_, home, about, posts, slides, other| {
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

        Ok(Page::file("sitemap.xml", String::from_utf8(buf).unwrap()))
    });

    let mut site = Site::new(site);

    match args.mode {
        Mode::Build => site.build(Global::new()),
        Mode::Watch => site.watch(Global::new()),
    }?;

    // let mut website = Website::config()
    //     .load_git(".")?
    //     .add_loaders([
    //         // github
    //         loader::async_asset("hauchiwa", async |_| {
    //             const URL: &str =
    //                 "https://raw.githubusercontent.com/kamoshi/hauchiwa/refs/heads/main/README.md";

    //             Ok(reqwest::get(URL).await?.text().await?)
    //         }),
    //     ])

    Ok(())
}
