use once_cell::sync::Lazy;
use regex::Regex;


static RE_RUBY: Lazy<Regex> = Lazy::new(||
    Regex::new(r"\[([^\]]+)\]\{([^}]+)\}").unwrap()
);

#[derive(Debug)]
pub(crate) enum Annotated<'a> {
    Text(&'a str),
    Ruby(&'a str, &'a str),
}


pub fn annotate(input: &str) -> Vec<Annotated> {
    let mut parts: Vec<Annotated> = Vec::new();
    let mut last_index = 0;

    for cap in RE_RUBY.captures_iter(input) {
        let text = cap.get(1).unwrap().as_str();
        let ruby = cap.get(2).unwrap().as_str();
        let index = cap.get(0).unwrap().start();

        if index > last_index {
            parts.push(Annotated::Text(&input[last_index..index]));
        }

        parts.push(Annotated::Ruby(text, ruby));
        last_index = cap.get(0).unwrap().end();
    }

    if last_index < input.len() {
        parts.push(Annotated::Text(&input[last_index..]));
    }

    parts
}
