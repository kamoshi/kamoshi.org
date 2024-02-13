mod captures;
mod configs;

use std::collections::HashMap;
use tree_sitter_highlight::Highlighter;
use tree_sitter_highlight::HighlightEvent;


#[macro_use]
extern crate napi_derive;


fn map_event(event: HighlightEvent, src: &str) -> HashMap<String, String> {
    match event {
        HighlightEvent::Source {start, end} => HashMap::from([
            ("kind".into(), "text".into()),
            ("text".into(), src[start..end].into())
        ]),
        HighlightEvent::HighlightStart(s) => HashMap::from([
            ("kind".into(), "open".into()),
            ("name".into(), captures::NAMES[s.0].into())
        ]),
        HighlightEvent::HighlightEnd => HashMap::from([
            ("kind".into(), "close".into())
        ]),
    }
}


#[napi]
pub fn hl(lang: String, src: String) -> Vec<HashMap<String, String>> {
    let config = match configs::get_config(&lang) {
        Some(c) => c,
        None => return vec![
            HashMap::from([
                ("kind".into(), "text".into()),
                ("text".into(), src.into())
            ])
        ]
    };


    let mut hl = Highlighter::new();
    let highlights = hl.highlight(
        &config,
        src.as_bytes(),
        None,
        |name| configs::get_config(name)
    ).unwrap();

    let mut out = vec![];
    for event in highlights {
        let event = event.unwrap();
        let obj = map_event(event, &src);
        out.push(obj);
    }
    out
}

