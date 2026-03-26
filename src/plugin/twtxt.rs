use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;
use minijinja::Value;

use crate::model::{Microblog, MicroblogEntry};
use crate::props::{PropsMicroblogEntry, PropsThought, PropsThoughts};
use crate::{Context, Global};

pub fn add_twtxt(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    styles: Many<Stylesheet>,
) -> Result<Many<Output>, HauchiwaError> {
    let twtxt = config.task().glob("content/twtxt.txt").map(|_, _, file| {
        let data = file.read()?;
        let data = String::from_utf8_lossy(&data);

        let entries = data
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                !line.is_empty() && !line.starts_with('#')
            })
            .map(str::parse::<MicroblogEntry>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Ok(Microblog {
            entries,
            data: data.to_string(),
        })
    })?;

    let handle = config
        .task()
        .using((templates, twtxt, styles))
        .spread(|ctx, (templates, twtxt, styles)| {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/microblog.scss")?,
            ];

            let data = twtxt.get("content/twtxt.txt")?;
            let html = render(ctx, templates, data, styles)?;

            let mut pages = vec![
                (
                    "twtxt.txt".into(),
                    Output::binary("twtxt.txt", data.data.clone()),
                ),
                ("thoughts".into(), Output::html("thoughts", html)),
            ];

            for entry in &data.entries {
                let html = render_entry(ctx, templates, entry, styles)?;
                let date = entry.date.timestamp();

                let path = format!("thoughts/{date}");
                pages.push((path.clone(), Output::html(path, html)));
            }

            Ok(pages)
        });

    Ok(handle)
}

fn make_entry_props(entry: &MicroblogEntry) -> PropsMicroblogEntry {
    let body = comrak::markdown_to_html(&entry.text, &comrak::Options::default());
    PropsMicroblogEntry {
        body: Value::from_safe_string(body),
        date_iso: entry.date.to_rfc3339(),
        date_display: entry.date.format("%b %d").to_string(),
        timestamp: entry.date.timestamp(),
    }
}

pub fn render(
    ctx: &Context,
    templates: &TemplateEnv,
    microblog: &Microblog,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let mut entries = microblog.entries.clone();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let props = PropsThoughts {
        head: super::make_props_head(ctx, "microblog".to_string(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        entries: entries.iter().map(make_entry_props).collect(),
    };

    let tmpl = templates.get_template("thoughts.jinja")?;
    Ok(tmpl.render(&props)?)
}

pub fn render_entry(
    ctx: &Context,
    templates: &TemplateEnv,
    entry: &MicroblogEntry,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    let props = PropsThought {
        head: super::make_props_head(ctx, "microblog".to_string(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        entry: make_entry_props(entry),
    };

    let tmpl = templates.get_template("thought.jinja")?;
    Ok(tmpl.render(&props)?)
}
