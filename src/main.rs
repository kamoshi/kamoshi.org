mod datalog;
mod md;
mod model;
mod plugin;
mod rss;
mod ts;
mod typst;

use std::fs;
use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::error::RuntimeError;
use hauchiwa::loader::image::{ImageFormat, Quality};
use hauchiwa::loader::sitemap::ChangeFrequency;
use hauchiwa::{Output, TaskContext, Website};
use hayagriva::Library;
use hypertext::{Raw, Renderable};

use crate::plugin::about::add_about;
use crate::plugin::home::add_home;
use crate::plugin::posts::add_posts;
use crate::plugin::projects::add_projects;
use crate::plugin::slides::add_slides;
use crate::plugin::tags::add_tags;
use crate::plugin::twtxt::add_twtxt;
use crate::plugin::wiki::add_teien;
use crate::plugin::{make_fullscreen, make_page};

/// Base path for content files
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
        .format(ImageFormat::Avif(Quality::Lossy(80)))
        .format(ImageFormat::WebP)
        .source("content/**/*.jpg")
        .source("content/**/*.png")
        .source("content/**/*.gif")
        .register()?;

    let styles = config
        .load_css()
        .entry("styles/**/[!_]*.scss")
        .watch("styles/**/*.scss")
        .minify(true)
        .register()?;

    let scripts = config
        .load_js()
        .entry("scripts/**/main.ts")
        .watch("scripts/**/*.ts")
        .minify(true)
        .register()?;

    let svelte = config
        .load_svelte::<()>()
        .entry("scripts/**/App.svelte")
        .watch("scripts/**/*.svelte")
        .register()?;

    let bibtex = config
        .task()
        .name("bibtex")
        .glob("content/**/*.bib")
        .map(|_, store, input| {
            let data = input.read()?;
            let path = store.save(&data, "bib")?;
            let text = String::from_utf8_lossy(&data);
            let data = hayagriva::io::from_biblatex_str(&text).unwrap();

            Ok(Bibtex { path, data })
        })?;

    // home
    let home = add_home(&mut config, images, styles, svelte)?;

    // about
    let about = add_about(&mut config, images, styles)?;

    // digital garden
    let teien = add_teien(&mut config, images, styles, bibtex)?;

    // twtxt
    let twtxt = add_twtxt(&mut config, styles)?;

    // posts
    let (posts_data, posts) = add_posts(&mut config, images, styles, scripts, bibtex)?;

    // slides
    let slides = add_slides(&mut config, images, styles, scripts)?;

    // projects
    let projects = add_projects(&mut config, styles)?;

    // tags
    let tags = add_tags(&mut config, posts_data, styles)?;

    // other
    let other = config
        .task()
        .name("other")
        .using((styles, scripts, svelte))
        .merge(|ctx, (styles, scripts, svelte)| {
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

    config
        .use_pagefind()
        .index(home)
        .index(about)
        .index(teien)
        .index(projects)
        .index(posts)
        .index(slides)
        .index(other)
        .register();

    config
        .use_sitemap(BASE_URL)
        .add(home, ChangeFrequency::Monthly, 0.8)
        .add(about, ChangeFrequency::Monthly, 0.8)
        .add(teien, ChangeFrequency::Monthly, 0.8)
        .add(twtxt, ChangeFrequency::Monthly, 0.8)
        .add(posts, ChangeFrequency::Monthly, 0.8)
        .add(slides, ChangeFrequency::Monthly, 0.8)
        .add(projects, ChangeFrequency::Monthly, 0.8)
        .add(tags, ChangeFrequency::Monthly, 0.8)
        .add(other, ChangeFrequency::Monthly, 0.8)
        .register();

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
