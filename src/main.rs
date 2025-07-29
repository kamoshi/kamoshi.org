mod hook;
mod markdown;
mod model;
mod plugin;
mod rss;
mod ts;
mod typst;

use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::loader::{self, Script, Svelte};
use hauchiwa::{Hook, Page, RuntimeError, Website};
use hayagriva::Library;
use hypertext::{Raw, Renderable};
use model::Slideshow;

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
pub struct Outline(pub Vec<(String, String)>);

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
            link: "https://git.kamoshi.org/kamov/kamoshi.org".into(),
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

    let mut website = Website::config()
        .load_git(".")?
        .add_plugins([
            plugin::home::PLUGIN,
            plugin::about::PLUGIN,
            plugin::posts::PLUGIN,
            plugin::slides::PLUGIN,
            plugin::projects::PLUGIN,
            plugin::wiki::PLUGIN,
            plugin::twtxt::PLUGIN,
            plugin::tags::PLUGIN,
        ])
        .add_loaders([
            // .bib -> Bibtex
            loader::glob_assets(CONTENT, "**/*.bib", |rt, data| {
                let path = rt.store(&data, "bib")?;
                let text = String::from_utf8_lossy(&data);
                let data = hayagriva::io::from_biblatex_str(&text).unwrap();

                Ok(Bibtex { path, data })
            }),
            // images
            loader::glob_images(CONTENT, "**/*.jpg"),
            loader::glob_images(CONTENT, "**/*.png"),
            loader::glob_images(CONTENT, "**/*.gif"),
            // svelte components
            loader::glob_svelte::<()>(CONTENT, "**/App.svelte"),
            loader::glob_svelte::<()>("scripts", "src/*/App.svelte"),
            // stylesheets
            loader::glob_styles("styles", "**/[!_]*.scss"),
            // scripts
            loader::glob_scripts("scripts", "src/*/main.ts"),
            // github
            loader::async_asset("hauchiwa", async |_| {
                const URL: &str =
                    "https://raw.githubusercontent.com/kamoshi/hauchiwa/refs/heads/main/README.md";

                Ok(reqwest::get(URL).await?.text().await?)
            }),
        ])
        .add_task("other", |ctx| {
            let mut pages = vec![];

            {
                let script = ctx.get::<Script>("src/photos/main.ts")?;
                let script = vec![script.path.to_string()];

                let html = Raw(r#"<div id="map" style="height: 100%; width: 100%"></div>"#);
                let html = make_fullscreen(&ctx, html, "Map".into(), script.into())?.render();

                pages.push(Page::html("map", html));
            }

            {
                const STYLES: &[&str] = &["styles.scss", "layouts/search.scss"];
                let Svelte { html, init } = ctx.get("src/search/App.svelte")?;
                let component = html(&())?;
                let script = vec![init.to_string()];

                let html = Raw(format!(r#"<main>{component}</main>"#));
                let html = make_page(&ctx, html, "Search".into(), STYLES, script.into())?.render();

                pages.push(Page::html("search", html));
            }

            Ok(pages)
        })
        .add_hook(Hook::post_build(crate::hook::build_pagefind))
        .add_hook(Hook::post_build(crate::hook::build_sitemap))
        .finish();

    match args.mode {
        Mode::Build => website.build(Global::new())?,
        Mode::Watch => website.watch(Global::new())?,
    };

    Ok(())
}
