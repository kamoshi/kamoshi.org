use std::{borrow::Cow, collections::HashMap, fmt::Write};

use camino::Utf8Path;
use comrak::{
    Arena, Options, format_html_with_plugins,
    nodes::{NodeHtmlBlock, NodeValue, NodeWikiLink},
    options::Plugins,
    parse_document,
};
use hauchiwa::loader::{Assets, Document};
use hypertext::Renderable;
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
