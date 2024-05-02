use std::collections::HashMap;

use hayagriva::{archive::ArchivedStyle, citationberg::{IndependentStyle, Locale, Style}, BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest, Library};
use hypertext::Renderable;
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use regex::Regex;

use crate::ts;


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

static LOCALE: Lazy<Vec<Locale>> = Lazy::new(|| hayagriva::archive::locales());

static STYLE: Lazy<IndependentStyle> = Lazy::new(||
    match ArchivedStyle::InstituteOfPhysicsNumeric.get() {
        Style::Independent(style) => style,
        Style::Dependent(_)       => unreachable!(),
    }
);

pub struct Outline(pub Vec<(String, String)>);


pub fn parse(text: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
    let (outline, stream) = {
        let stream = Parser::new_ext(text, *OPTS);
        let mut stream: Vec<_> = TextMergeStream::new(stream).collect();
        let outline = set_heading_ids(&mut stream);
        (outline, stream)
    };

    let stream = stream.into_iter()
        .map(make_math)
        .map(make_emoji)
        .collect::<Vec<_>>();

    let stream = make_code(stream)
        .into_iter()
        .flat_map(make_ruby)
        .flat_map(make_cite)
        .collect::<Vec<_>>();

    let (stream, bib) = match lib {
        Some(lib) => make_bib(stream, lib),
        None      => (stream, None),
    };

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, stream.into_iter());

    (outline, html, bib)
}

fn make_bib<'a, 'b>(stream: Vec<Event<'a>>, lib: &'b Library) -> (Vec<Event<'a>>, Option<Vec<String>>) {
    let mut driver = BibliographyDriver::new();

    for event in stream.iter() {
        match event {
            Event::InlineMath(ref text) => match lib.get(text) {
                Some(entry) => driver.citation(CitationRequest::from_items(vec![CitationItem::with_entry(entry)], &STYLE, &LOCALE)),
                None        => (),
            },
            _ => (),
        }
    }

    // add fake citation to make all entries show up
    driver.citation(CitationRequest::from_items(lib.iter().map(CitationItem::with_entry).collect(), &STYLE, &LOCALE));

    let res = driver.finish(BibliographyRequest { style: &STYLE, locale: None, locale_files: &LOCALE });

    let mut n = 0;
    let stream = stream.into_iter()
        .map(|event| match event {
            Event::InlineMath(name) => {
                let mut buffer = String::from("<cite>");
                match res.citations.get(n) {
                    Some(rf) => rf.citation.write_buf(&mut buffer, BufWriteFormat::Html).unwrap(),
                    None     => buffer.push_str(&name),
                };
                buffer.push_str("</cite>");
                n += 1;
                Event::InlineHtml(buffer.into())
            },
            _ => event
        })
        .collect();

    let bib = res.bibliography.map(|bib|
        bib.items.iter()
            .map(|x| {
                let mut buffer = String::new();
                x.content.write_buf(&mut buffer, BufWriteFormat::Html).unwrap();
                buffer
            })
            .collect::<Vec<_>>()
    );

    (stream, bib)
}

static RE_CITE: Lazy<Regex> = Lazy::new(|| Regex::new(r":cite\[([^\]]+)\]").unwrap());

#[derive(Debug)]
enum Annotated_<'a> {
    Text(&'a str),
    Cite(&'a str),
}

fn annotate_(input: &str) -> Vec<Annotated_> {
    let mut parts: Vec<Annotated_> = Vec::new();
    let mut last_index = 0;

    for cap in RE_CITE.captures_iter(input) {
        let cite = cap.get(1).unwrap().as_str();
        let index = cap.get(0).unwrap().start();

        if index > last_index {
            parts.push(Annotated_::Text(&input[last_index..index]));
        }

        parts.push(Annotated_::Cite(cite));
        last_index = cap.get(0).unwrap().end();
    }

    if last_index < input.len() {
        parts.push(Annotated_::Text(&input[last_index..]));
    }

    parts
}

fn make_cite(event: Event) -> Vec<Event> {
    match event {
        Event::Text(ref text) => {
            annotate_(text)
                .into_iter()
                .map(|e| match e {
                    Annotated_::Text(text) => Event::Text(text.to_owned().into()),
                    Annotated_::Cite(cite) => Event::InlineMath(cite.to_owned().into()),
                })
                .collect()
        },
        _ => vec![event],
    }
}

fn set_heading_ids(events: &mut [Event]) -> Outline {
    let mut cnt = HashMap::<String, i32>::new();
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut ptr = None;

    for event in events {
        match event {
            Event::Start(ref mut tag @ Tag::Heading {..}) => {
                ptr = Some(tag);
            },
            Event::Text(ref text) if ptr.is_some() => {
                buf.push_str(text)
            },
            Event::End(TagEnd::Heading(..)) => {
                let txt = std::mem::take(&mut buf);
                let url = txt.to_lowercase().replace(' ', "-");
                let url = match cnt.get_mut(&url) {
                    Some(ptr) => { *ptr += 1; format!("{url}-{ptr}") },
                    None      => { cnt.insert(url.clone(), 0); url },
                };
                match ptr.take().unwrap() {
                    Tag::Heading { ref mut id, .. } => *id = Some(url.clone().into()),
                    _ => unreachable!(),
                }
                out.push((txt, url));
            },
            _ => (),
        }
    };

    Outline(out)
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

static RE_RUBY: Lazy<Regex> = Lazy::new(||
    Regex::new(r"\[([^\]]+)\]\{([^}]+)\}").unwrap()
);

#[derive(Debug)]
enum Annotated<'a> {
    Text(&'a str),
    Ruby(&'a str, &'a str),
}


fn annotate(input: &str) -> Vec<Annotated> {
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

fn make_ruby(event: Event) -> Vec<Event> {
    match event {
        Event::Text(ref text) => annotate(text)
            .into_iter()
            .map(|el| match el {
                Annotated::Text(text) => Event::Text(text.to_owned().into()),
                Annotated::Ruby(t, f) => Event::InlineHtml(format!("<ruby>{t}<rp>(</rp><rt>{f}</rt><rp>)</rp></ruby>").into()),
            })
            .collect(),
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
                    top = idx + 1;
                }

                old = idx + 1;
            }

            if let Some(ref mut buf) = buf {
                buf.push_str(&text[top..]);
            }

            match buf {
                None      => event,
                Some(buf) => Event::Text(buf.into())
            }
        },
        _ => event,
    }
}
