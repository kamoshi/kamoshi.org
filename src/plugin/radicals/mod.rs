use hauchiwa::{
    Blueprint, Handle, Output,
    error::HauchiwaError,
    loader::{Assets, Stylesheet, Svelte},
};
use hypertext::{Raw, Renderable};

use crate::{Global, plugin::make_fullscreen};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Props {
    url: String,
}

pub fn build(
    config: &mut Blueprint<Global>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Output>, HauchiwaError> {
    let svelte =
        config.load_svelte::<Props>("src/plugin/radicals/App.svelte", "src/plugin/radicals/")?;

    let radicals = config.load("src/plugin/radicals/kradfile.gz", |_, store, input| {
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Read;

        use encoding_rs::EUC_JP;
        use flate2::read::GzDecoder;

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

                kanji_radicals.insert(kanji.to_string(), radicals);
            }
        }

        let data = serde_json::to_string(&kanji_radicals)?;
        let path = store.save(data.as_bytes(), "json")?;

        Ok(path)
    })?;

    Ok(hauchiwa::task!(config, |ctx, styles, svelte, radicals| {
        let Svelte {
            prerender,
            hydration,
            ..
        } = svelte.get("src/plugin/radicals/App.svelte")?;

        let props = Props {
            url: radicals.get("src/plugin/radicals/kradfile.gz")?.to_string(),
        };

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/radicals.scss")?,
        ];

        let scripts = &[hydration];

        let html = make_fullscreen(
            ctx,
            Raw::dangerously_create(format!(r#"<main>{}</main>"#, prerender(&props)?)),
            "Radicals".into(),
            styles,
            scripts,
        )?
        .render()
        .into_inner();

        Ok(Output::html("radicals", html))
    }))
}
