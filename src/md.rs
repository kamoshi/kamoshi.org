use std::{borrow::Cow, collections::HashMap, fmt::Write, sync::LazyLock};

use camino::Utf8Path;
use comrak::{
    Arena, Node, Options, format_html_with_plugins,
    nodes::{NodeHtmlBlock, NodeValue, NodeWikiLink},
    options::Plugins,
    parse_document,
};
use hauchiwa::loader::{Assets, Document};
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

    options.render.r#unsafe = true;

    options
}

fn get_plugins() -> Plugins<'static> {
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&Highlighter);

    plugins
}

pub fn parse_markdown(
    md: &str,
    resolver: &WikiLinkResolver,
) -> Result<(String, Vec<String>), MarkdownError> {
    let arena = Arena::new();

    let options = get_options();
    let plugins = get_plugins();

    let root = parse_document(&arena, md, &options);

    // First, process ruby annotations
    // [text]{ruby} -> <ruby><rb>text</rb><rp>(</rp><rt>ruby</rt><rp>)</rp></ruby>
    process_ruby(&arena, &root);

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
                let url = resolver.resolve(&link.url)?;

                refs.push(url.clone());
                data.value = NodeValue::WikiLink(NodeWikiLink { url });
            }
            _ => {}
        }
    }

    let mut html = String::new();
    format_html_with_plugins(root, &options, &mut html, &plugins)?;

    Ok((html, refs))
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

    pub fn from_assets<T>(prefix: &str, assets: &Assets<Document<T>>) -> Self
    where
        T: Clone,
    {
        let mut resolver = Self::new();
        resolver.add_all(prefix, assets);
        resolver
    }

    pub fn add<T>(&mut self, prefix: &str, doc: &Document<T>)
    where
        T: Clone,
    {
        let url = doc.href(prefix);
        let path = Utf8Path::new(&url);
        // "wiki/tech/rust.html" -> "rust"
        if let Some(stem) = path.file_stem() {
            self.index
                .entry(stem.to_lowercase())
                .or_default()
                .push(url.to_string());
        }
    }

    pub fn add_all<T>(&mut self, prefix: &str, docs: &Assets<Document<T>>)
    where
        T: Clone,
    {
        for doc in docs.values() {
            self.add(prefix, doc);
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
