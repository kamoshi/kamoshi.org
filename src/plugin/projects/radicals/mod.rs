use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;

use encoding_rs::EUC_JP;
use flate2::read::GzDecoder;
use hauchiwa::{
    Blueprint, Handle, Output,
    error::HauchiwaError,
    loader::{Assets, Stylesheet, Svelte},
};
use hypertext::{Raw, Renderable};

use crate::plugin::make_bare;
use crate::{Global, plugin::make_fullscreen};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Props {
    url: String,
}

const CHARS: &str = include_str!("./kklc.txt");

pub fn build(
    config: &mut Blueprint<Global>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Output>, HauchiwaError> {
    let svelte = config.load_svelte::<Props>(
        "src/plugin/projects/radicals/App.svelte",
        "src/plugin/projects/radicals/",
    )?;

    let radicals = config.load(
        "src/plugin/projects/radicals/kradfile.gz",
        |_, store, input| {
            let set = CHARS.chars().map(|c| c.to_string()).collect::<HashSet<_>>();

            let mut kanji_radicals: HashMap<String, Vec<String>> = HashMap::new();

            let file = File::open(&input.path)?;
            let mut decoder = GzDecoder::new(file);

            // Read the raw bytes (because we need to decode EUC-JP manually)
            let mut buffer = Vec::new();
            decoder.read_to_end(&mut buffer)?;

            // Decode EUC-JP to UTF-8 String
            // .0 contains the Cow<str>, .1 is the encoding used, .2 is boolean for errors
            let (content, _, _) = EUC_JP.decode(&buffer);

            // --- 3. Parse lines ---
            for line in content.lines() {
                let line = line.trim();

                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Split by " : "
                if let Some((kanji, radicals_part)) = line.split_once(" : ") {
                    let radicals: Vec<String> = radicals_part
                        .split_whitespace() // Handles space separation
                        .map(|s| s.to_string())
                        .collect();

                    if set.contains(kanji) {
                        kanji_radicals.insert(kanji.to_string(), radicals);
                    }
                }
            }

            let data = serde_json::to_string(&kanji_radicals)?;
            let path = store.save(data.as_bytes(), "json")?;

            Ok(path)
        },
    )?;

    Ok(hauchiwa::task!(config, |ctx, styles, svelte, radicals| {
        let Svelte {
            prerender,
            hydration,
            ..
        } = svelte.get("src/plugin/projects/radicals/App.svelte")?;

        let props = Props {
            url: radicals
                .get("src/plugin/projects/radicals/kradfile.gz")?
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
    }))
}
