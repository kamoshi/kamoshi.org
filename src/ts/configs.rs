use std::collections::HashMap;
use once_cell::sync::Lazy;
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

macro_rules! insert {
    ($_:tt $e:expr) => { $e };
}

macro_rules! merge {
    ([$($e:expr),+ $(,)?]) => { &format!(concat!($(insert!($e "{} ")),*), $($e),* ) };
    ($e:expr)              => { $e };
}

macro_rules! language {
    ($name:expr, $lang:expr, $highlights:expr, $injections:expr, $locals:expr $(,)?) => {
        (
            $name,
            {
                let mut config = HighlightConfiguration::new(
                    $lang,
                    $name,
                    $highlights,
                    $injections,
                    $locals,
                ).unwrap();
                config.configure(captures::NAMES);
                config
            }
        )
    };
}

pub static EXTENSIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("hs", "haskell"),
        ("js", "javascript"),
        ("md", "markdown"),
        ("mdx", "markdown"),
        ("py", "python"),
        ("scm", "scheme"),
        ("ts", "javascript"),
        ("typescript", "javascript")
    ])
});


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
        language!(
            "css",
            tree_sitter_css::language(),
            tree_sitter_css::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        language!(
            "haskell",
            tree_sitter_haskell::language(),
            tree_sitter_haskell::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_haskell::LOCALS_QUERY,
        ),
        language!(
            "html",
            tree_sitter_html::language(),
            tree_sitter_html::HIGHLIGHTS_QUERY,
            tree_sitter_html::INJECTIONS_QUERY,
            "",
        ),
        language!(
            "javascript",
            tree_sitter_javascript::language(),
            merge!([
                query!("ecma/highlights"),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
            ]),
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        language!(
            "jsx",
            tree_sitter_javascript::language(),
            merge!([
                query!("ecma/highlights"),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
            ]),
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        language!(
            "markdown",
            tree_sitter_md::language(),
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            tree_sitter_md::INJECTION_QUERY_BLOCK,
            "",
        ),
        language!(
            "python",
            tree_sitter_python::language(),
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        language!(
            "regex",
            tree_sitter_regex::language(),
            query!("regex/highlights"),
            "",
            "",
        ),
        language!(
            "rust",
            tree_sitter_rust::language(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        ),
        language!(
            "scss",
            tree_sitter_scss::language(),
            merge!([
                tree_sitter_css::HIGHLIGHTS_QUERY,
                tree_sitter_scss::HIGHLIGHTS_QUERY,
            ]),
            "",
            "",
        ),
        language!(
            "query",
            tree_sitter_query::language(),
            tree_sitter_query::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        language!(
            "toml",
            tree_sitter_toml_ng::language(),
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        language!(
            "typescript",
            tree_sitter_typescript::language_typescript(),
            merge!([
                query!("ecma/highlights"),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            ]),
            tree_sitter_javascript::INJECTIONS_QUERY,
            merge!([
                tree_sitter_javascript::LOCALS_QUERY,
                tree_sitter_typescript::LOCALS_QUERY,
            ])
        ),
        language!(
            "tsx",
            tree_sitter_typescript::language_tsx(),
            merge!([
                query!("ecma/highlights"),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            ]),
            tree_sitter_javascript::INJECTIONS_QUERY,
            merge!([
                tree_sitter_javascript::LOCALS_QUERY,
                tree_sitter_typescript::LOCALS_QUERY,
            ]),
        ),
    ])
});


pub fn get_config(name: &str) -> Option<&'static HighlightConfiguration> {
    match EXTENSIONS.get(name) {
        Some(name) => CONFIGS.get(name),
        None => CONFIGS.get(name),
    }
}
