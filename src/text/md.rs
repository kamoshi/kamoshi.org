use hypertext::Renderable;
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::ts;

use super::ruby;


static OPTS: Lazy<Options> = Lazy::new(||
    Options::empty()
        .union(Options::ENABLE_MATH)
        .union(Options::ENABLE_TABLES)
        .union(Options::ENABLE_TASKLISTS)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_SMART_PUNCTUATION)
);

static KATEX_I: Lazy<katex::Opts> = Lazy::new(||
    katex::opts::Opts::builder()
        .output_type(katex::OutputType::Mathml)
        .build()
        .unwrap()
);

static KATEX_B: Lazy<katex::Opts> = Lazy::new(||
    katex::opts::Opts::builder()
        .output_type(katex::OutputType::Mathml)
        .display_mode(true)
        .build()
        .unwrap()
);


pub fn parse(text: &str) -> String {
    let stream = Parser::new_ext(text, *OPTS)
        .map(make_math)
        .map(make_emoji)
        .collect::<Vec<_>>();

    let stream = make_code(stream)
        .into_iter()
        .flat_map(make_ruby);

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, stream.into_iter());
    html
}

fn make_math(event: Event) -> Event {
    match event {
        Event::InlineMath(math)  => Event::InlineHtml(katex::render_with_opts(&math, &*KATEX_I).unwrap().into()),
        Event::DisplayMath(math) => Event::Html(katex::render_with_opts(&math, &*KATEX_B).unwrap().into()),
        _ => event
    }
}

fn make_code(es: Vec<Event>) -> Vec<Event> {
    let mut buff = Vec::new();
    let mut lang = None;
    let mut code = String::new();

    for event in es {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => match kind {
                CodeBlockKind::Indented     => (),
                CodeBlockKind::Fenced(name) => lang = Some(name),
            },
            Event::End(TagEnd::CodeBlock) => {
                let lang = lang.take().unwrap_or("".into());
                let html = ts::highlight(&lang, &code).render().as_str().to_owned();
                buff.push(Event::Html(html.into()));
                code.clear();
            },
            Event::Text(text) => match lang {
                None    => buff.push(Event::Text(text)),
                Some(_) => code.push_str(&text),
            },
            _ => buff.push(event)
        }
    }

    buff
}

fn make_ruby(event: Event) -> Vec<Event> {
    match event {
        Event::Text(text) => {
            let mut buff = Vec::new();

            for item in ruby::annotate(&text) {
                match item {
                    ruby::Annotated::Text(text) => buff.push(Event::Text(text.to_owned().into())),
                    ruby::Annotated::Ruby(t, f) => buff.push(Event::InlineHtml(format!("<ruby>{t}<rp>(</rp><rt>{f}</rt><rp>)</rp></ruby>").into())),
                };
            }

            buff
        },
        _ => vec![event],
    }
}

fn make_emoji(event: Event) -> Event {
    match event {
        Event::Text(ref text) => {
            let mut buf = None;
            let mut top = 0;
            let mut old = 0;

            for (idx, _) in text.match_indices(':') {
                let key = &text[old..idx];

                if let Some(emoji) = emojis::get_by_shortcode(key) {
                    let buf = buf.get_or_insert_with(|| String::with_capacity(text.len()));
                    buf.push_str(&text[top..old-1]);
                    buf.push_str(emoji.as_str());
                    top = idx;
                }

                old = idx + 1;
            }

            match buf {
                None      => event,
                Some(buf) => Event::Text(buf.into())
            }
        },
        _ => event,
    }
}
