use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tree_sitter_highlight::HighlightConfiguration;


pub const NAMES: &[&str] = &[
    "comment",

    "attribute",
    "carriage-return",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "constructor.builtin",
    "embedded",
    "error",
    "escape",
    "function",
    "function.builtin",
    "include",
    "keyword",
    "markup",
    "markup.bold",
    "markup.heading",
    "markup.italic",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.list.checked",
    "markup.list.numbered",
    "markup.list.unchecked",
    "markup.list.unnumbered",
    "markup.quote",
    "markup.raw",
    "markup.raw.block",
    "markup.raw.inline",
    "markup.strikethrough",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];


pub static CONFIGS: Lazy<HashMap<&'static str, Arc<HighlightConfiguration>>> = Lazy::new(|| {
    [
        (
            vec!["css", "scss"],
            HighlightConfiguration::new(
                tree_sitter_css::language(),
                tree_sitter_css::HIGHLIGHTS_QUERY,
                "",
                "",
            ).unwrap()
        ),
        (
            vec!["hs", "haskell"],
            HighlightConfiguration::new(
                tree_sitter_haskell::language(),
                tree_sitter_haskell::HIGHLIGHTS_QUERY,
                "",
                tree_sitter_haskell::LOCALS_QUERY,
            ).unwrap()
        ),
        (
            vec!["html", "html"],
            HighlightConfiguration::new(
                tree_sitter_html::language(),
                tree_sitter_html::HIGHLIGHTS_QUERY,
                tree_sitter_html::INJECTIONS_QUERY,
                "",
            ).unwrap()
        ),
        (
            vec!["md", "markdown"],
            HighlightConfiguration::new(
                tree_sitter_md::language(),
                // &format!("{}\n\n{}",
                    tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                //     tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
                // ),
                // &format!("{}\n\n{}",
                    tree_sitter_md::INJECTION_QUERY_BLOCK,
                //     tree_sitter_md::INJECTION_QUERY_INLINE,
                // ),
                ""
            ).unwrap()
        ),
        (
            vec!["rs", "rust"],
            HighlightConfiguration::new(
                tree_sitter_rust::language(),
                tree_sitter_rust::HIGHLIGHT_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            ).unwrap()
        ),
        (
            vec!["js", "javascript"],
            HighlightConfiguration::new(
                tree_sitter_javascript::language(),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            ).unwrap()
        ),
        (
            vec!["jsx"],
            HighlightConfiguration::new(
                tree_sitter_javascript::language(),
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            ).unwrap()
        ),
        (
            vec!["ts", "typescript"],
            HighlightConfiguration::new(
                tree_sitter_typescript::language_typescript(),
                &format!("{}\n\n{}",
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_typescript::HIGHLIGHT_QUERY
                ),
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_typescript::LOCALS_QUERY,
            ).unwrap()
        ),
    ]
        .into_iter()
        .flat_map(|(keys, mut config)| {
            config.configure(NAMES);
            let config = Arc::new(config);
            keys.into_iter().map(move |key| (key, config.clone()))
        })
        .collect()
});
