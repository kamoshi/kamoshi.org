use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

use camino::Utf8Path;
use hayagriva::{
    BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest,
    Library,
    archive::ArchivedStyle,
    citationberg::{IndependentStyle, Locale, Style},
};
use hypertext::Renderable;
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, Options as OptsMarkdown, Parser, Tag, TagEnd, TextMergeStream,
};
use regex::Regex;

use crate::{Bibliography, Context, Outline, ts};

const OPTS_MARKDOWN: OptsMarkdown = OptsMarkdown::empty()
    .union(OptsMarkdown::ENABLE_MATH)
    .union(OptsMarkdown::ENABLE_TABLES)
    .union(OptsMarkdown::ENABLE_TASKLISTS)
    .union(OptsMarkdown::ENABLE_STRIKETHROUGH)
    .union(OptsMarkdown::ENABLE_SMART_PUNCTUATION);

fn render_directive_inline(name: &str, content: &str) -> Event<'static> {
    match name {
        "icon" => Event::InlineHtml(format!(r#"<img class="inline-icon" src="{content}">"#).into()),
        "cite" => {
            // iff math has been already rendered we can repurpose the nodes to store citations
            Event::InlineMath(content.to_owned().into())
        }
        other => panic!("Unknown inline directive {other}"),
    }
}

fn render_directive_block(name: &str, content: &str) -> Event<'static> {
    match name {
        "youtube" => {
            let iframe = format!(
                "<iframe width='560' height='315' src='https://www.youtube.com/embed/{}' frameborder='0' allowfullscreen></iframe>",
                content
            );
            Event::Html(iframe.into())
        }
        other => panic!("Unknown block directive {other}"),
    }
}

fn render_directive_container(name: &str, events: Vec<Event>) -> Event<'static> {
    match name {
        "sidenote" => {
            let mut parsed = String::new();
            pulldown_cmark::html::push_html(&mut parsed, events.into_iter());
            Event::Html(format!("<aside class='marginnote'>{parsed}</aside>").into())
        }
        other => panic!("Unknown block directive {other}"),
    }
}

pub fn parse(
    content: &str,
    sack: &Context,
    path: &Utf8Path,
    library: Option<&Library>,
) -> (String, Outline, Bibliography) {
    let mut outline = vec![];

    let stream = Parser::new_ext(content, OPTS_MARKDOWN);
    let stream = TextMergeStream::new(stream);

    let stream = StreamHeading::new(stream, &mut outline);
    let stream = StreamCodeBlock::new(stream);
    let stream = stream.map(swap_hashed_image(path, sack));
    let stream = stream.map(render_latex);
    let stream = stream.map(render_emoji);
    let stream = StreamRuby::new(stream);
    let stream = StreamDirectiveContainer::new(stream, render_directive_container);
    let stream = StreamDirectiveBlock::new(stream, render_directive_block);
    let stream = StreamDirectiveInline::new(stream, render_directive_inline);

    let (events, bibliography) = match library {
        Some(library) => make_bib(Vec::from_iter(stream), library),
        None => (Vec::from_iter(stream), None),
    };

    let mut parsed = String::new();
    pulldown_cmark::html::push_html(&mut parsed, events.into_iter());

    (parsed, Outline(outline), Bibliography(bibliography))
}

// StreamHeading

struct StreamHeading<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    iter: I,
    counts: HashMap<String, i32>,
    buffer: String,
    handle: Option<Tag<'a>>,
    events: VecDeque<Event<'a>>,
    finish: bool,
    out: &'a mut Vec<(String, String)>,
}

impl<'a, I> StreamHeading<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    pub fn new(iter: I, out: &'a mut Vec<(String, String)>) -> Self {
        Self {
            iter,
            counts: HashMap::new(),
            buffer: String::new(),
            handle: None,
            events: VecDeque::new(),
            finish: false,
            out,
        }
    }
}

impl<'a, I> Iterator for StreamHeading<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.finish && !self.events.is_empty() {
            true => return self.events.pop_front(),
            false => self.finish = false,
        }

        for event in self.iter.by_ref() {
            match event {
                Event::Start(tag @ Tag::Heading { .. }) => {
                    debug_assert!(self.handle.is_none());
                    self.handle = Some(tag);
                }
                Event::Text(text) if self.handle.is_some() => {
                    self.buffer.push_str(&text);
                    self.events.push_back(Event::Text(text));
                }
                event @ Event::End(TagEnd::Heading(..)) => {
                    debug_assert!(self.handle.is_some());
                    self.events.push_back(event);

                    let txt = std::mem::take(&mut self.buffer);
                    let mut url = txt.to_lowercase().replace(' ', "-");

                    match self.counts.get_mut(&url) {
                        Some(count) => {
                            *count += 1;
                            url = format!("{url}-{count}");
                        }
                        None => {
                            self.counts.insert(url.clone(), 0);
                        }
                    }

                    let mut handle = self.handle.take().unwrap();
                    match handle {
                        Tag::Heading { ref mut id, .. } => *id = Some(url.clone().into()),
                        _ => unreachable!(),
                    }

                    self.out.push((txt, url.clone()));
                    self.finish = true;
                    return Some(Event::Start(handle));
                }
                _ => return Some(event),
            }
        }
        None
    }
}

// StreamCodeBlock

pub struct StreamCodeBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    iter: I,
    lang: Option<String>,
    code: String,
}

impl<'a, I> StreamCodeBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            lang: None,
            code: String::new(),
        }
    }
}

impl<'a, I> Iterator for StreamCodeBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for event in self.iter.by_ref() {
            match &event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Indented => {
                            // return indented code blocks as-is
                            return Some(event);
                        }
                        CodeBlockKind::Fenced(name) => {
                            // capture language to highlight
                            self.lang = Some(name.to_string());
                        }
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    // end of code block, process the collected code
                    let lang = self.lang.take().unwrap_or_else(|| "".into());
                    let html = ts::highlight(&lang, &self.code)
                        .render()
                        .as_str()
                        .to_owned();
                    self.code.clear(); // Clear buffer for the next block
                    return Some(Event::Html(html.into())); // Emit HTML event
                }
                Event::Text(text) => match self.lang.is_some() {
                    true => self.code.push_str(text), // -> collect text into code buffer if inside a code block
                    false => return Some(event),      // -> pass through text outside code blocks
                },
                _ => {
                    // Pass through other events unchanged
                    if self.lang.is_none() {
                        return Some(event);
                    }
                }
            }
        }
        None
    }
}

// Swap hashed image

fn swap_hashed_image<'a>(dir: &'a Utf8Path, sack: &'a Context) -> impl Fn(Event<'a>) -> Event<'a> {
    move |event| match event {
        Event::Start(start) => match start {
            Tag::Image {
                dest_url,
                link_type,
                title,
                id,
            } => {
                let rel = dir.join(dest_url.as_ref());
                let img = sack.get_image(rel.as_str());
                let hashed = img.map(|path| path.as_str().to_owned().into());
                Event::Start(Tag::Image {
                    link_type,
                    dest_url: hashed.unwrap_or(dest_url),
                    title,
                    id,
                })
            }
            _ => Event::Start(start),
        },
        _ => event,
    }
}

// LaTeX

fn parse_latex(math: &str, block: bool) -> String {
    use pulldown_latex::*;

    let config = RenderConfig {
        display_mode: match block {
            true => config::DisplayMode::Block,
            false => config::DisplayMode::Inline,
        },
        ..Default::default()
    };

    let storage = Storage::new();
    let parser = Parser::new(&math, &storage);
    let mut buffer = String::new();
    push_mathml(&mut buffer, parser, config).expect("MathML fail");
    buffer
}

fn render_latex(event: Event) -> Event {
    match event {
        Event::InlineMath(math) => Event::InlineHtml(parse_latex(&math, false).into()),
        Event::DisplayMath(math) => Event::Html(parse_latex(&math, true).into()),
        event => event,
    }
}

// Emojis

fn render_emoji(event: Event) -> Event {
    match event {
        Event::Text(ref text) => {
            let mut buf = None;
            let mut top = 0;
            let mut old = 0;

            for (idx, _) in text.match_indices(':') {
                let key = &text[old..idx];

                if let Some(emoji) = emojis::get_by_shortcode(key) {
                    let buf = buf.get_or_insert_with(|| String::with_capacity(text.len()));
                    buf.push_str(&text[top..old - 1]);
                    buf.push_str(emoji.as_str());
                    top = idx + 1;
                }

                old = idx + 1;
            }

            if let Some(ref mut buf) = buf {
                buf.push_str(&text[top..]);
            }

            match buf {
                None => event,
                Some(buf) => Event::Text(buf.into()),
            }
        }
        _ => event,
    }
}

// StreamDirectiveContainer

// The regexes are defined without end-of-line anchors so we can split if they occur in the middle.
static RE_DIRECTIVE_CONTAINER_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:::+\s*(\w*)\s*$").unwrap());
static RE_DIRECTIVE_CONTAINER_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:::+\s*$").unwrap());

/// The iterator adapter which processes directive containers.
struct StreamDirectiveContainer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    inner: I,
    /// A stack to hold "split‐off" events that need to be returned.
    queue: VecDeque<Event<'a>>,
    /// The current state: either not in a container or accumulating a container.
    state: Option<(String, Vec<Event<'a>>)>,
    /// The callback to process the container block.
    callback: fn(&str, Vec<Event<'a>>) -> Event<'static>,
}

impl<'a, I> StreamDirectiveContainer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(inner: I, callback: fn(&str, Vec<Event<'a>>) -> Event<'static>) -> Self {
        Self {
            inner,
            queue: VecDeque::new(),
            state: None,
            callback,
        }
    }
}

impl<'a, I> Iterator for StreamDirectiveContainer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return any pending events from the stack first.
        if let Some(event) = self.queue.pop_front() {
            return Some(event);
        }

        while let Some(event) = self.inner.next() {
            match event {
                Event::Text(line) => match &mut self.state {
                    // Normal mode: look for a directive start marker.
                    None => {
                        if let Some(m) = RE_DIRECTIVE_CONTAINER_START.find(&line) {
                            let start_idx = m.start();
                            let end_idx = m.end();

                            // Split the current line into three parts:
                            // 1. Text before the marker.
                            // 2. The marker text.
                            // 3. Text after the marker.
                            let before = &line[..start_idx];
                            let marker = &line[start_idx..end_idx];
                            let after = &line[end_idx..];

                            // Process the marker to extract an identifier, if provided.
                            if let Some(captures) = RE_DIRECTIVE_CONTAINER_START.captures(marker) {
                                let ident = captures
                                    .get(1)
                                    .map(|m| m.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let mut stack = Vec::new();

                                // If there’s text after the marker, push it inside.
                                if !after.is_empty() {
                                    stack.push(Event::Text(after.to_string().into()));
                                }

                                self.state = Some((ident, stack));
                            }

                            // If there’s text before the marker, we can return it instantly!
                            match before.is_empty() {
                                false => return Some(Event::Text(before.to_string().into())),
                                true => continue,
                            }
                        } else {
                            // No marker found, return the event as-is.
                            return Some(Event::Text(line));
                        }
                    }
                    // Inside a directive block: accumulate events until the end marker is found.
                    Some((ident, events)) => {
                        if let Some(m) = RE_DIRECTIVE_CONTAINER_END.find(&line) {
                            let start_idx = m.start();
                            let end_idx = m.end();

                            // Split this line into text before the marker, the marker, and after.
                            let before = &line[..start_idx];
                            let after = &line[end_idx..];

                            // Append any text before the marker to our accumulated events.
                            if !before.is_empty() {
                                events.push(Event::Text(before.to_string().into()));
                            }

                            // The marker indicates the end of the directive container.
                            // Invoke the callback with the collected events.
                            let directive_event =
                                (self.callback)(ident, events.drain(..).collect());

                            // Reset our state back to normal.
                            self.state = None;

                            // If there is text after the end marker, push it to the stack.
                            if !after.is_empty() {
                                self.queue.push_back(Event::Text(after.to_string().into()));
                            }

                            // Return the event produced by the callback.
                            // (In this example the callback returns an Event<'static>.
                            // To match the iterator’s lifetime, we use unsafe transmute;
                            // in real code you may want to design the types to avoid this.)
                            return Some(unsafe { std::mem::transmute(directive_event) });
                        } else {
                            // No end marker found on this line – accumulate it.
                            events.push(Event::Text(line));
                            // Continue the loop to read the next event.
                            continue;
                        }
                    }
                },
                // For non-text events, if we're inside a directive, accumulate them.
                event => match &mut self.state {
                    // inside container
                    Some((_, vec)) => {
                        vec.push(event);
                        continue;
                    }
                    // outside container
                    None => return Some(event),
                },
            }
        }

        self.queue.pop_front()
    }
}

// StreamDirectiveBlock

static RE_DIRECTIVE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^::(\w+)\[(.*?)\]$").unwrap());

struct StreamDirectiveBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    inner: I,
    callback: fn(&str, &str) -> Event<'static>,
}

impl<'a, I> StreamDirectiveBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(inner: I, callback: fn(&str, &str) -> Event<'static>) -> Self {
        Self { inner, callback }
    }
}

impl<'a, I> Iterator for StreamDirectiveBlock<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.inner.next()?;

        if let Event::Text(line) = &event {
            if let Some(captures) = RE_DIRECTIVE_BLOCK.captures(line) {
                let name = captures.get(1).unwrap().as_str();
                let content = captures.get(2).unwrap().as_str();
                return Some((self.callback)(name, content));
            }
        }

        Some(event)
    }
}

// StreamDirectiveInline

static RE_DIRECTIVE_INLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":(\w+)\[(.*?)\]").unwrap());

struct StreamDirectiveInline<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    inner: I,
    state: Option<(usize, CowStr<'a>)>,
    callback: fn(&str, &str) -> Event<'static>,
}

impl<'a, I> StreamDirectiveInline<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(inner: I, callback: fn(&str, &str) -> Event<'static>) -> Self {
        Self {
            inner,
            state: None,
            callback,
        }
    }
}

impl<'a, I> Iterator for StreamDirectiveInline<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.is_none() {
            match self.inner.next()? {
                Event::Text(str) => self.state = Some((0, str)),
                event => return Some(event),
            }
        };

        let (idx, str) = self.state.clone()?;
        let slice = &str[idx..];

        if let Some(mat) = RE_DIRECTIVE_INLINE.find(slice) {
            if mat.start() > 0 {
                let (idx, str) = self.state.take().unwrap();
                self.state = Some((idx + mat.start(), str));
                let returned = &slice[..mat.start()];
                return Some(Event::Text(returned.to_owned().into()));
            }

            let captures = RE_DIRECTIVE_INLINE.captures(slice).unwrap();
            let name = captures.get(1).unwrap().as_str();
            let content = captures.get(2).unwrap().as_str();

            let directive = (self.callback)(name, content);

            let (idx, str) = self.state.take().unwrap();
            self.state = Some((idx + mat.end(), str));
            return Some(directive);
        }

        let (idx, str) = self.state.take()?;
        Some(Event::Text(str[idx..].to_owned().into()))
    }
}

// Render citations

static LOCALE: LazyLock<Vec<Locale>> = LazyLock::new(hayagriva::archive::locales);

static STYLE: LazyLock<IndependentStyle> =
    LazyLock::new(
        || match ArchivedStyle::InstituteOfElectricalAndElectronicsEngineers.get() {
            Style::Independent(style) => style,
            Style::Dependent(_) => unreachable!(),
        },
    );

fn make_bib<'a>(
    stream: Vec<Event<'a>>,
    library: &Library,
) -> (Vec<Event<'a>>, Option<Vec<String>>) {
    let mut driver = BibliographyDriver::new();

    for event in stream.iter() {
        if let Event::InlineMath(text) = event {
            if let Some(entry) = library.get(text) {
                driver.citation(CitationRequest::from_items(
                    vec![CitationItem::with_entry(entry)],
                    &STYLE,
                    &LOCALE,
                ))
            }
        }
    }

    // add fake citation to make all entries show up
    driver.citation(CitationRequest::from_items(
        library.iter().map(CitationItem::with_entry).collect(),
        &STYLE,
        &LOCALE,
    ));

    let res = driver.finish(BibliographyRequest {
        style: &STYLE,
        locale: None,
        locale_files: &LOCALE,
    });

    let mut n = 0;
    let stream = stream
        .into_iter()
        .map(|event| match event {
            Event::InlineMath(name) => {
                let mut buffer = String::from("<cite>");
                match res.citations.get(n) {
                    Some(rf) => rf
                        .citation
                        .write_buf(&mut buffer, BufWriteFormat::Html)
                        .unwrap(),
                    None => buffer.push_str(&name),
                };
                buffer.push_str("</cite>");
                n += 1;
                Event::InlineHtml(buffer.into())
            }
            _ => event,
        })
        .collect();

    let bib = res.bibliography.map(|bib| {
        bib.items
            .iter()
            .map(|x| {
                let mut buffer = String::new();
                x.content
                    .write_buf(&mut buffer, BufWriteFormat::Html)
                    .unwrap();
                buffer
            })
            .collect::<Vec<_>>()
    });

    (stream, bib)
}

// Render Ruby

static RE_RUBY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\{([^}]+)\}").unwrap());

struct StreamRuby<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    inner: I,
    state: Option<(usize, CowStr<'a>)>,
}

impl<'a, I> StreamRuby<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(inner: I) -> Self {
        Self { inner, state: None }
    }
}

impl<'a, I> Iterator for StreamRuby<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((idx, str)) = self.state.take() {
            // if there is any ruby left in text
            if let Some(capture) = RE_RUBY.captures(&str[idx..]) {
                let full_match = capture.get(0).unwrap();

                // if there is outstanding text before ruby
                if full_match.start() > 0 {
                    let idx_next = idx + full_match.start();
                    let prefix = String::from(&str[idx..idx_next]);
                    self.state = Some((idx_next, str));
                    return Some(Event::Text(prefix.into()));
                }

                let text = capture.get(1).unwrap().as_str();
                let ruby = capture.get(2).unwrap().as_str();
                let ruby_html = format!("<ruby>{text}<rp>(</rp><rt>{ruby}</rt><rp>)</rp></ruby>");

                self.state = Some((idx + full_match.end(), str));
                return Some(Event::InlineHtml(ruby_html.into()));
            }

            // return any remaining text
            if idx < str.len() {
                let remaining = String::from(&str[idx..]);
                return Some(Event::Text(remaining.into()));
            }
        }

        match self.inner.next()? {
            Event::Text(str) => {
                self.state = Some((0, str));
                self.next()
            }
            event => Some(event),
        }
    }
}
