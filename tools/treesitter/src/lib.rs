#![deny(clippy::all)]
mod configs;

use std::collections::HashMap;
use tree_sitter_highlight::Highlighter;
use tree_sitter_highlight::HighlightEvent;
use configs::{CONFIGS, NAMES};


#[macro_use]
extern crate napi_derive;


#[napi]
pub fn hl(lang: String, src: String) -> Vec<HashMap<String, String>> {
    let config = match CONFIGS.get(&*lang) {
        Some(c) => c,
        None => return vec![
            HashMap::from([
                ("kind".into(), "text".into()),
                ("text".into(), src.into())
            ])
        ]
    };


    let mut highlighter = Highlighter::new();
    let highlights = highlighter.highlight(
        &config,
        src.as_bytes(),
        None,
        |key| CONFIGS.get(key).map(|arc| arc.as_ref())
    ).unwrap();

    let mut out = vec![];
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::Source {start, end} => out.push(HashMap::from([
                ("kind".into(), "text".into()),
                ("text".into(), src[start..end].into())
            ])),
            HighlightEvent::HighlightStart(s) => out.push(HashMap::from([
                ("kind".into(), "open".into()),
                ("name".into(), NAMES[s.0].into())
            ])),
            HighlightEvent::HighlightEnd => out.push(HashMap::from([
                ("kind".into(), "close".into())
            ]))
        }
    }

    out
}

