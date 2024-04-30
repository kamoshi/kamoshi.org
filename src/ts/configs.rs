use std::collections::HashMap;
use once_cell::sync::Lazy;
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

use super::captures;


macro_rules! query {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/queries/",
            $path,
            ".scm"
        ))
    };
}

pub static EXTENSIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("hs", "haskell"),
        ("js", "javascript"),
        ("md", "markdown"),
        ("mdx", "markdown"),
        ("py", "python"),
        ("query", "query"),
        // ("scm", "scheme"),
        ("scss", "scss"),
        ("ts", "javascript"),
        ("typescript", "javascript")
    ])
});

fn config_for(
    lang: Language,
    name: &str,
    highlights: &str,
    injections: &str,
    locals: &str,
) -> HighlightConfiguration {
    let mut config = HighlightConfiguration::new(lang, name, highlights, injections, locals).unwrap();
    config.configure(captures::NAMES);
    config
}

pub static CONFIGS: Lazy<HashMap<&'static str, HighlightConfiguration>> = Lazy::new(|| {
    HashMap::from([
        // (
        //     "astro",
        //     config_for(
        //         tree_sitter_astro::language(),
        //         query!("astro/highlights"),
        //         query!("astro/injections"),
        //         "",
        //     )
        // ),
        // (
        //     "css",
        //     config_for(
        //         tree_sitter_css::language(),
        //         query!("css/highlights"),
        //         "",
        //         "",
        //     )
        // ),
        (
            "haskell",
            config_for(
                npezza93_tree_sitter_haskell::language(),
                "haskell",
                npezza93_tree_sitter_haskell::HIGHLIGHTS_QUERY,
                "",
                npezza93_tree_sitter_haskell::LOCALS_QUERY,
            )
        ),
        // (
        //     "html",
        //     config_for(
        //         tree_sitter_html::language(),
        //         tree_sitter_html::HIGHLIGHTS_QUERY,
        //         tree_sitter_html::INJECTIONS_QUERY,
        //         "",
        //     )
        // ),
        (
            "javascript",
            config_for(
                tree_sitter_javascript::language(),
                "javascript",
                &format!("{} {}",
                    query!("ecma/highlights"),
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                ),
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
        ),
        (
            "jsx",
            config_for(
                tree_sitter_javascript::language(),
                "jsx",
                &format!("{} {} {}",
                    query!("ecma/highlights"),
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                ),
                tree_sitter_javascript::INJECTION_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
        ),
        (
            "markdown",
            config_for(
                tree_sitter_md::language(),
                "markdown",
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                tree_sitter_md::INJECTION_QUERY_BLOCK,
                "",
            )
        ),
        (
            "python",
            config_for(
                tree_sitter_python::language(),
                "python",
                tree_sitter_python::HIGHLIGHTS_QUERY,
                "",
                "",
            )
        ),
        (
            "query",
            config_for(
                tree_sitter_query::language(),
                "query",
                tree_sitter_query::HIGHLIGHTS_QUERY,
                tree_sitter_query::INJECTIONS_QUERY,
                "",
            )
        ),
        // (
        //     "regex",
        //     config_for(
        //         tree_sitter_regex::language(),
        //         query!("regex/highlights"),
        //         "",
        //         "",
        //     )
        // ),
        (
            "rust",
            config_for(
                tree_sitter_rust::language(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
        ),
        // (
        //     "scheme",
        //     config_for(
        //         tree_sitter_scheme::language(),
        //         tree_sitter_scheme::HIGHLIGHTS_QUERY,
        //         "",
        //         "",
        //     )
        // ),
        // (
        //     "toml",
        //     config_for(
        //         tree_sitter_toml::language(),
        //         tree_sitter_toml::HIGHLIGHT_QUERY,
        //         "",
        //         "",
        //     )
        // ),
        // (
        //     "tsx",
        //     config_for(
        //         tree_sitter_typescript::language_tsx(),
        //         &format!("{} {} {} {}",
        //             query!("ecma/highlights"),
        //             tree_sitter_javascript::HIGHLIGHT_QUERY,
        //             tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
        //             tree_sitter_typescript::HIGHLIGHT_QUERY,
        //         ),
        //         tree_sitter_javascript::INJECTION_QUERY,
        //         &format!("{} {}",
        //             tree_sitter_javascript::LOCALS_QUERY,
        //             tree_sitter_typescript::LOCALS_QUERY
        //         )
        //     )
        // ),
        // (
        //     "typescript",
        //     config_for(
        //         tree_sitter_typescript::language_typescript(),
        //         &format!("{} {} {}",
        //             query!("ecma/highlights"),
        //             tree_sitter_javascript::HIGHLIGHT_QUERY,
        //             tree_sitter_typescript::HIGHLIGHT_QUERY,
        //         ),
        //         tree_sitter_javascript::INJECTION_QUERY,
        //         &format!("{} {}",
        //             tree_sitter_javascript::LOCALS_QUERY,
        //             tree_sitter_typescript::LOCALS_QUERY
        //         ),
        //     )
        // ),
    ])
});


pub fn get_config(name: &str) -> Option<&'static HighlightConfiguration> {
    match EXTENSIONS.get(name) {
        Some(name) => CONFIGS.get(name),
        None => CONFIGS.get(name),
    }
}
