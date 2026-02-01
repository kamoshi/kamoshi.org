use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

use hauchiwa::prelude::*;
use hauchiwa::{
    Blueprint, Output,
    error::HauchiwaError,
    loader::{Stylesheet, Svelte},
};
use hypertext::{Raw, Renderable};

use crate::Global;
use crate::plugin::make_bare;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Props {
    url: String,
}

const CHARS: &str = include_str!("./kklc.txt");

pub fn build(
    config: &mut Blueprint<Global>,
    styles: Many<Stylesheet>,
) -> Result<One<Output>, HauchiwaError> {
    let svelte = config
        .load_svelte::<Props>()
        .entry("src/plugin/projects/radicals/App.svelte")
        .watch("src/plugin/projects/radicals/")
        .register()?;

    let radicals = config
        .task()
        .source("src/plugin/projects/radicals/IDS.TXT")
        .run(|_, store, input| {
            // 1. Prepare the filter set
            let set = CHARS.chars().map(|c| c.to_string()).collect::<HashSet<_>>();
            let mut kanji_components: HashMap<String, Vec<String>> = HashMap::new();

            // 2. Open file (UTF-8 default)
            let file = File::open(&input.path)?;
            let reader = BufReader::new(file);

            // 3. Parse lines
            for line in reader.lines() {
                let line = line?;
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // IDS.TXT format: U+CODE [TAB] KANJI [TAB] IDS_1 [TAB] IDS_2 ...
                let parts: Vec<&str> = line.split('\t').collect();

                if parts.len() < 3 {
                    continue;
                }

                let kanji = parts[1];

                // Only process if it's in our kklc.txt list
                if set.contains(kanji) {
                    // --- SELECTING THE BEST IDS SEQUENCE ---
                    // We look for a sequence tagged with 'J' (Japan).
                    // If none found, we default to the first one (parts[2]).
                    let raw_ids = parts[2..]
                        .iter()
                        .find(|&s| s.contains("(J"))
                        .unwrap_or(&parts[2]); // Fallback to generic if no 'J' tag

                    // --- CLEANING THE SEQUENCE ---
                    // 1. Remove structure operators (U+2FF0 to U+2FFB like ⿰, kw)
                    // 2. Remove format markers (^, $) and tags like (G), (J)
                    let components: Vec<String> = raw_ids
                        .chars()
                        .filter(|c| !is_ids_operator(*c)) // Remove ⿰, etc.
                        .filter(|c| *c != '^' && *c != '$') // Remove markers
                        // Stop processing if we hit the tags starting with '('
                        .take_while(|c| *c != '(')
                        .map(|c| c.to_string())
                        .collect();

                    kanji_components.insert(kanji.to_string(), components);
                }
            }

            let data = serde_json::to_string(&kanji_components)?;
            let path = store.save(data.as_bytes(), "json")?;

            Ok(path)
        })?;

    let task = config.task().depends_on((styles, svelte, radicals)).run(
        |ctx, (styles, svelte, radicals)| {
            let Svelte {
                prerender,
                hydration,
                ..
            } = svelte.get("src/plugin/projects/radicals/App.svelte")?;

            let props = Props {
                url: radicals
                    .get("src/plugin/projects/radicals/IDS.TXT")?
                    .to_string(),
            };

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/radicals.scss")?,
            ];

            let scripts = &[hydration];

            let html = make_bare(
                ctx,
                Raw::dangerously_create(format!(r#"<main>{}</main>"#, prerender(&props)?)),
                "Radicals".into(),
                styles,
                scripts,
            )?
            .render()
            .into_inner();

            Ok(Output::html("projects/radicals", html))
        },
    );

    Ok(task)
}

// Helper to identify the "operator" characters
fn is_ids_operator(c: char) -> bool {
    // The unicode block for Ideographic Description Characters is U+2FF0 to U+2FFB
    matches!(c, '\u{2FF0}'..='\u{2FFB}')
}
