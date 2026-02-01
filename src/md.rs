use std::{borrow::Cow, collections::HashMap, fmt::Write, sync::LazyLock};

use camino::Utf8Path;
use comrak::{
    Arena, Node, Options, format_html_with_plugins,
    nodes::{NodeHtmlBlock, NodeValue, NodeWikiLink},
    options::Plugins,
    parse_document,
};
use hauchiwa::{
    Tracker,
    loader::{Document, Image, generic::DocumentMeta},
};
use hypertext::Renderable;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownError {
    #[error("Link target not found: '{0}'")]
    WikiLinkNotFound(String),

    #[error("Ambiguous link '{0}'. matches multiple candidates: {1:?}")]
    WikiLinkAmbiguous(String, Vec<String>),

    #[error("Formatting error")]
    Format(#[from] std::fmt::Error),
}

pub struct Highlighter;

impl comrak::adapters::SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::fmt::Result {
        let lang = lang.unwrap_or("text");
        let html = crate::ts::highlight(lang, code).render().into_inner();

        write!(output, "{}", html)?;

        Ok(())
    }

    fn write_pre_tag(&self, _: &mut dyn Write, _: HashMap<&str, Cow<str>>) -> std::fmt::Result {
        Ok(())
    }

    fn write_code_tag(&self, _: &mut dyn Write, _: HashMap<&str, Cow<str>>) -> std::fmt::Result {
        Ok(())
    }
}

fn get_options() -> Options<'static> {
    let mut options = Options::default();

    options.parse.smart = true;

    options.extension.math_dollars = true;
    options.extension.shortcodes = true;
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.wikilinks_title_after_pipe = true;
    options.extension.header_ids = Some("".into());

    options.render.r#unsafe = true;

    options
}

fn get_plugins() -> Plugins<'static> {
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&Highlighter);

    plugins
}

pub struct Parsed {
    pub html: String,
    pub refs: Vec<String>,
    pub outline: Outline,
    pub bibliography: Option<Vec<String>>,
}

pub fn parse(
    file_text: &str,
    file_meta: &DocumentMeta,
    resolver: Option<&WikiLinkResolver>,
    images: Option<&Tracker<Image>>,
    library: Option<&hayagriva::Library>,
) -> Result<Parsed, MarkdownError> {
    let arena = Arena::new();

    let options = get_options();
    let plugins = get_plugins();

    let root = parse_document(&arena, file_text, &options);

    // Process ruby annotations
    // [text]{ruby} -> <ruby><rb>text</rb><rp>(</rp><rt>ruby</rt><rp>)</rp></ruby>
    process_ruby(&arena, &root);

    // Process images
    // ![alt](path) -> <figure><picture>...</picture><figcaption>alt</figcaption></figure>
    process_images(file_meta, images, &arena, &root)?;

    let mut refs = Vec::new();

    for node in root.descendants() {
        let mut data = node.data.borrow_mut();

        match &data.value {
            NodeValue::Math(math) => {
                let text = &math.literal;
                let is_display = math.display_math;

                let math = parse_latex(text, is_display);

                if is_display {
                    data.value = NodeValue::HtmlBlock(NodeHtmlBlock {
                        block_type: 0,
                        literal: math,
                    });
                } else {
                    data.value = NodeValue::HtmlInline(math);
                }
            }
            NodeValue::WikiLink(link) => {
                if let Some(resolver) = resolver {
                    let url = resolver.resolve(&link.url)?;

                    refs.push(url.clone());
                    data.value = NodeValue::WikiLink(NodeWikiLink { url });
                }
            }
            _ => {}
        }
    }

    process_inline_directives(&arena, &root);

    let mut bibliography = None;

    if let Some(library) = library {
        bibliography = process_citations(library, &root)?;
    }

    let outline = process_headings(&arena, &root);

    let mut html = String::new();
    format_html_with_plugins(root, &options, &mut html, &plugins)?;

    Ok(Parsed {
        html,
        refs,
        outline,
        bibliography,
    })
}

// hashed images

fn process_images<'arena, 'a>(
    file_meta: &'a DocumentMeta,
    images: Option<&'a Tracker<Image>>,
    arena: &'a Arena<'arena>,
    root: &'a Node<'arena>,
) -> std::fmt::Result
where
    'a: 'arena,
{
    let mut nodes = Vec::new();

    for node in root.descendants() {
        if let NodeValue::Image(image) = &node.data.borrow().value {
            nodes.push((node, image.clone()));
        }
    }

    for (node, link) in nodes {
        if let Some(images) = images
            && let Some(image) = resolve_image_path(file_meta, &link.url, images)
        {
            // Create a temporary Document node to act as a container for alt
            // text nodes and render them to HTML
            let caption = {
                let root = arena.alloc(NodeValue::Document.into());

                for child in node.children() {
                    child.detach();
                    root.append(child);
                }

                let mut caption = String::new();
                comrak::format_html(root, &comrak::Options::default(), &mut caption)?;

                caption
            };

            let literal = render_picture(image, &caption);
            let html = arena.alloc(
                NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 0,
                    literal,
                })
                .into(),
            );

            let mut target = node;

            // check if the node is a child of a paragraph
            if let Some(parent) = node.parent()
                && matches!(parent.data.borrow().value, NodeValue::Paragraph)
                && parent.children().count() == 1
            {
                target = parent;
            }

            target.insert_before(html);
            target.detach();
        }
    }

    Ok(())
}

fn resolve_image_path<'ctx, 'a>(
    file_meta: &'a DocumentMeta,
    text_url: &'a str,
    images: &'ctx Tracker<Image>,
) -> Option<&'ctx Image> {
    // Skip absolute URLs (http://...)
    if text_url.contains("://") {
        return None;
    }

    let relative = file_meta.resolve(text_url);

    images.get(&relative).ok()
}

fn render_picture(image: &Image, alt: &str) -> String {
    use hypertext::prelude::*;

    maud!(
        figure {
            picture {
                @for path in image.sources.values() {
                    source srcset=(path.as_str());
                }
                img
                    alt=""
                    src=(image.default.as_str())
                    width=(image.width)
                    height=(image.height);
            }
            figcaption {
                (alt)
            }
        }
    )
    .render()
    .into_inner()
}

// math

fn parse_latex(math: &str, is_display: bool) -> String {
    use pulldown_latex::config::DisplayMode;
    use pulldown_latex::{Parser, RenderConfig, Storage, push_mathml};

    let config = RenderConfig {
        display_mode: if is_display {
            DisplayMode::Block
        } else {
            DisplayMode::Inline
        },
        ..Default::default()
    };

    let storage = Storage::new();
    let parser = Parser::new(math, &storage);

    let mut buffer = String::new();
    push_mathml(&mut buffer, parser, config).expect("MathML fail");

    buffer
}

// wikilink

pub struct WikiLinkResolver {
    index: HashMap<String, Vec<String>>,
}

impl WikiLinkResolver {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn from_assets<T>(assets: &Tracker<Document<T>>) -> Self
    where
        T: Clone,
    {
        let mut resolver = Self::new();
        resolver.add_all(assets);
        resolver
    }

    pub fn add<T>(&mut self, doc: &Document<T>)
    where
        T: Clone,
    {
        let href = doc.meta.href.as_str();
        let path = Utf8Path::new(href);
        // "wiki/tech/rust.html" -> "rust"
        if let Some(stem) = path.file_stem() {
            self.index
                .entry(stem.to_lowercase())
                .or_default()
                .push(href.to_string());
        }
    }

    pub fn add_all<T>(&mut self, docs: &Tracker<Document<T>>)
    where
        T: Clone,
    {
        for doc in docs {
            self.add(doc);
        }
    }

    pub fn resolve(&self, link: &str) -> Result<String, MarkdownError> {
        // Extract stem (e.g., "a/b/Note" -> "note")
        let link_path = Utf8Path::new(link);

        let stem = link_path
            .file_stem()
            .map(|s| s.to_lowercase())
            .ok_or_else(|| MarkdownError::WikiLinkNotFound(link.to_string()))?;

        // Check if any file with this name exists
        let candidates = self
            .index
            .get(&stem)
            .ok_or_else(|| MarkdownError::WikiLinkNotFound(link.to_string()))?;

        // Filter candidates that match the explicit path provided in the link
        // e.g. [[tech/Rust]] matches "/wiki/tech/Rust.html" but not "/wiki/game/Rust.html"
        let matches: Vec<String> = candidates
            .iter()
            .filter(|candidate| {
                Utf8Path::new(candidate)
                    .with_extension("")
                    .ends_with(link_path)
            })
            .cloned()
            .collect();

        match matches.as_slice() {
            [] => Err(MarkdownError::WikiLinkNotFound(link.to_string())),
            [it] => Ok(it.clone()), // Perfect match
            _ => Err(MarkdownError::WikiLinkAmbiguous(link.to_string(), matches)),
        }
    }
}

// ruby matcher
static RE_RUBY: LazyLock<Regex> = LazyLock::new(|| {
    // Matches [Kanji]{kana}
    Regex::new(r"\[([^\]]+)\]\{([^}]+)\}").expect("Invalid Ruby Regex")
});

// This traverses the AST and replaces matching Text nodes with Ruby HTML nodes
fn process_ruby<'arena, 'a>(arena: &'a Arena<'arena>, root: &'a Node<'arena>)
where
    'a: 'arena,
{
    // We store the Node, the Text, and the slices of the Ruby matches.
    let mut nodes_to_modify = Vec::new();

    // Identify candidates and calculate all slices.
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();

        if let NodeValue::Text(text) = &mut data.value {
            // Verify matches and collect indices in one go.
            let matches: Vec<_> = RE_RUBY
                .captures_iter(text)
                .map(|cap| {
                    (
                        cap.get(0).unwrap().range(), // Full match range: [Kanji]{kana}
                        cap.get(1).unwrap().range(), // Kanji range
                        cap.get(2).unwrap().range(), // Kana range
                    )
                })
                .collect();

            // If we found ruby, we steal the text (mem::take) and queue it up.
            if !matches.is_empty() {
                nodes_to_modify.push((node, std::mem::take(text), matches));
            }
        }
    }

    // Now we just need to go through the nodes_to_modify and replace the raw
    // text with new text nodes interspersed with Ruby HTML.
    for (node, text, matches) in nodes_to_modify {
        let mut last_idx = 0;

        for (full_range, kanji_range, kana_range) in matches {
            // Insert preceding text
            if full_range.start > last_idx {
                let pre_text = &text[last_idx..full_range.start];
                let pre_node = arena.alloc(NodeValue::Text(pre_text.to_string().into()).into());
                node.insert_before(pre_node);
            }

            // Insert Ruby HTML
            let kanji = &text[kanji_range];
            let kana = &text[kana_range];

            let ruby_html = format!(
                "<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>",
                kanji, kana
            );

            let ruby_node = arena.alloc(NodeValue::HtmlInline(ruby_html).into());
            node.insert_before(ruby_node);

            last_idx = full_range.end;
        }

        // Insert remaining text
        if last_idx < text.len() {
            let post_text = &text[last_idx..];
            let post_node = arena.alloc(NodeValue::Text(post_text.to_string().into()).into());
            node.insert_before(post_node);
        }

        // Detach empty node
        node.detach();
    }
}

// inline directive

static RE_DIRECTIVE_INLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":(\w+)\[(.*?)\]").expect("Invalid regex"));

fn render_directive_inline<'a>(arena: &'a Arena<'a>, name: String, content: String) -> Node<'a> {
    match name.as_str() {
        "icon" => {
            let html = format!(r#"<img class="inline-icon" src="{content}">"#);
            arena.alloc(NodeValue::HtmlInline(html).into())
        }
        "cite" => {
            // Repurpose Math node for citations
            arena.alloc(
                NodeValue::Math(comrak::nodes::NodeMath {
                    dollar_math: false,
                    display_math: false,
                    literal: content.to_string(),
                })
                .into(),
            )
        }
        _ => {
            // Fallback: If unknown, perhaps render as plain text or a warning
            let fallback = format!(":{name}[{content}]");
            arena.alloc(NodeValue::Text(fallback.into()).into())
        }
    }
}

fn process_inline_directives<'arena, 'a>(arena: &'a Arena<'arena>, root: &'a Node<'arena>)
where
    'a: 'arena,
{
    let mut nodes_to_modify = Vec::new();

    // Scan for directives in all Text nodes
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Text(text) = &mut data.value {
            let matches: Vec<_> = RE_DIRECTIVE_INLINE
                .captures_iter(text)
                .map(|cap| {
                    (
                        cap.get(0).unwrap().range(),              // Full match: :name[content]
                        cap.get(1).unwrap().as_str().to_string(), // Name
                        cap.get(2).unwrap().as_str().to_string(), // Content
                    )
                })
                .collect();

            if !matches.is_empty() {
                nodes_to_modify.push((node, std::mem::take(text), matches));
            }
        }
    }

    // Apply transformations
    for (node, text, matches) in nodes_to_modify {
        let mut last_idx = 0;

        for (range, name, content) in matches {
            // Push text occurring before the directive
            if range.start > last_idx {
                let pre_text = &text[last_idx..range.start];
                let pre_node = arena.alloc(NodeValue::Text(pre_text.to_string().into()).into());
                node.insert_before(pre_node);
            }

            let directive_node = render_directive_inline(arena, name, content);

            node.insert_before(directive_node);
            last_idx = range.end;
        }

        // Push any remaining text after the last directive
        if last_idx < text.len() {
            let post_text = &text[last_idx..];
            let post_node = arena.alloc(NodeValue::Text(post_text.to_string().into()).into());
            node.insert_before(post_node);
        }

        // Remove the original text node
        node.detach();
    }
}

// citations

static LOCALE: LazyLock<Vec<hayagriva::citationberg::Locale>> =
    LazyLock::new(hayagriva::archive::locales);

static STYLE: LazyLock<hayagriva::citationberg::IndependentStyle> = LazyLock::new(|| {
    match hayagriva::archive::ArchivedStyle::InstituteOfElectricalAndElectronicsEngineers.get() {
        hayagriva::citationberg::Style::Independent(style) => style,
        hayagriva::citationberg::Style::Dependent(_) => unreachable!(),
    }
});

fn process_citations<'arena, 'a>(
    library: &hayagriva::Library,
    root: &'a Node<'arena>,
) -> Result<Option<Vec<String>>, std::fmt::Error> {
    use hayagriva::{
        BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest,
    };

    // Check math nodes that exist
    // We store the Node reference and the key to avoid borrowing issues later
    let mut candidates = Vec::new();

    for node in root.descendants() {
        if let NodeValue::Math(math) = &node.data.borrow().value {
            // Only process inline math that matches a library key
            if !math.display_math && library.get(&math.literal).is_some() {
                candidates.push((node, math.literal.clone()));
            }
        }
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    let mut driver = BibliographyDriver::new();

    // Register each citation found in the text
    for (_, key) in &candidates {
        if let Some(entry) = library.get(key) {
            driver.citation(CitationRequest::from_items(
                vec![CitationItem::with_entry(entry)],
                &STYLE,
                &LOCALE,
            ));
        }
    }

    // This ensures *every* item in the library appears in the final
    // bibliography list, not just the ones cited in the text.
    driver.citation(CitationRequest::from_items(
        library.iter().map(CitationItem::with_entry).collect(),
        &STYLE,
        &LOCALE,
    ));

    // Render results
    let result = driver.finish(BibliographyRequest {
        style: &STYLE,
        locale: None,
        locale_files: &LOCALE,
    });

    // Swap Math nodes for HTML <cite> tags. Skip the last entry because that
    // corresponds to the "fake pass" that includes all items.
    for ((node, _), item) in candidates.iter().zip(result.citations.iter()) {
        let mut buf = String::from("<cite>");

        item.citation.write_buf(&mut buf, BufWriteFormat::Html)?;
        buf.push_str("</cite>");

        node.data.borrow_mut().value = NodeValue::HtmlInline(buf);
    }

    let mut bibliography = Vec::new();

    if let Some(entries) = result.bibliography {
        for item in entries.items {
            let mut buf = String::new();

            item.content.write_buf(&mut buf, BufWriteFormat::Html)?;

            bibliography.push(buf);
        }
    }

    Ok(Some(bibliography))
}

// outline

#[derive(Debug, Clone)]
pub struct Heading {
    pub title: String,
    pub id: String,
    pub children: Vec<Heading>,
}

#[derive(Debug, Clone)]
pub struct Outline(pub Vec<Heading>);

impl From<Vec<(String, String, usize)>> for Outline {
    fn from(flat_vec: Vec<(String, String, usize)>) -> Self {
        let mut res = Vec::<Heading>::new();

        for (title, url, level) in flat_vec {
            let mut ptr = &mut res;

            let new = Heading {
                title,
                id: url,
                children: vec![],
            };

            for _ in 2..level {
                // This fixes the borrow checker issue
                if ptr.is_empty() {
                    break;
                }

                // We can unwrap here because we checked if the vector is empty
                ptr = &mut ptr.last_mut().unwrap().children;
            }

            ptr.push(new);
        }

        Outline(res)
    }
}

impl hypertext::Renderable for Outline {
    fn render_to(&self, buffer: &mut hypertext::Buffer<hypertext::context::Node>) {
        use hypertext::prelude::*;

        fn render_heading_list(headings: &[Heading], depth: usize) -> impl Renderable {
            maud!(
                ul class=(format!("outline-depth-{depth}")) {
                    @for Heading { title, id, children } in headings {
                        li {
                            a href=(format!("#{id}")) { (title) }

                            @if !children.is_empty() {
                                (render_heading_list(children, depth + 1))
                            }
                        }
                    }
                }
            )
        }

        maud!(
            aside .outline {
                @if !self.0.is_empty() {
                    section {
                        h2 {
                            a href="#top" { "Outline" }
                        }
                        nav #table-of-contents {
                            (render_heading_list(&self.0, 1))
                        }
                    }
                }
            }
        )
        .render_to(buffer);
    }
}

fn process_headings<'a>(arena: &'a Arena<'a>, root: &'a Node<'a>) -> Outline {
    let mut flat_headings = Vec::new();
    let mut counts = HashMap::new();

    let mut nodes_to_process = Vec::new();

    for node in root.descendants() {
        if let NodeValue::Heading(heading) = node.data.borrow().value {
            nodes_to_process.push((node, heading.level));
        }
    }

    for (node, level) in nodes_to_process {
        let text = extract_text(&node);

        let mut slug = text.to_lowercase().replace(' ', "-");
        match counts.get_mut(&slug) {
            Some(count) => {
                *count += 1;
                slug = format!("{slug}-{count}");
            }
            None => {
                counts.insert(slug.clone(), 0);
            }
        }

        // We create a temporary root to render just the children of this heading
        let inner_html = {
            let root = arena.alloc(NodeValue::Document.into());

            for child in node.children() {
                child.detach();
                root.append(child);
            }

            let mut html = String::new();
            comrak::format_html(root, &comrak::Options::default(), &mut html).unwrap();

            html
        };

        // Manually build the <hX id="..."> tag
        let html = format!("<h{level} id=\"{slug}\">{inner_html}</h{level}>");

        let new_node = arena.alloc(
            NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 0,
                literal: html,
            })
            .into(),
        );

        // Swap the nodes
        node.insert_before(new_node);
        node.detach();

        // Add to Outline
        flat_headings.push((text, slug, level as usize));
    }

    Outline::from(flat_headings)
}

/// Helper to recursively extract text from a node's children
fn extract_text(node: &Node) -> String {
    let mut buf = String::new();

    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => buf.push_str(t),
            NodeValue::Code(c) => buf.push_str(&c.literal),
            _ => buf.push_str(&extract_text(&child)),
        }
    }

    buf
}
