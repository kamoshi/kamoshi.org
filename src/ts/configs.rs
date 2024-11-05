use std::collections::HashMap;
use std::sync::LazyLock;

use tree_sitter_highlight::HighlightConfiguration;

use crate::ts::captures;

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
	($_:tt $str:literal) => {
		$str
	};
}

macro_rules! merge {
    [$($any:expr),+ $(,)?] => {
        &format!(concat!($(insert!($any "{} ")),*), $($any),* )
    };
}

macro_rules! language {
	($name:expr, $lang:expr, $highlights:expr, $injections:expr, $locals:expr $(,)?) => {
		($name, {
			let mut config =
				HighlightConfiguration::new($lang, $name, $highlights, $injections, $locals)
					.unwrap();
			config.configure(captures::NAMES);
			config
		})
	};
}

static EXTENSIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
	HashMap::from([
		("hs", "haskell"),
		("js", "javascript"),
		("md", "markdown"),
		("mdx", "markdown"),
		("ml", "ocaml"),
		("mli", "ocaml_interface"),
		("py", "python"),
		("scm", "scheme"),
		("ts", "typescript"),
		("typescript", "javascript"),
	])
});

static CONFIGS: LazyLock<HashMap<&'static str, HighlightConfiguration>> = LazyLock::new(|| {
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
			tree_sitter_css::LANGUAGE.into(),
			tree_sitter_css::HIGHLIGHTS_QUERY,
			"",
			"",
		),
		language!(
			"haskell",
			tree_sitter_haskell::LANGUAGE.into(),
			tree_sitter_haskell::HIGHLIGHTS_QUERY,
			tree_sitter_haskell::INJECTIONS_QUERY,
			tree_sitter_haskell::LOCALS_QUERY,
		),
		language!("html", tree_sitter_html::LANGUAGE.into(), "", "", "",),
		language!(
			"javascript",
			tree_sitter_javascript::LANGUAGE.into(),
			merge![
				query!("ecma/highlights"),
				tree_sitter_javascript::HIGHLIGHT_QUERY,
			],
			tree_sitter_javascript::INJECTIONS_QUERY,
			tree_sitter_javascript::LOCALS_QUERY,
		),
		language!(
			"jsx",
			tree_sitter_javascript::LANGUAGE.into(),
			merge![
				query!("ecma/highlights"),
				tree_sitter_javascript::HIGHLIGHT_QUERY,
				tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
			],
			tree_sitter_javascript::INJECTIONS_QUERY,
			tree_sitter_javascript::LOCALS_QUERY,
		),
		language!(
			"markdown",
			tree_sitter_md::LANGUAGE.into(),
			tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
			tree_sitter_md::INJECTION_QUERY_BLOCK,
			"",
		),
		language!(
			"markdown_inline",
			tree_sitter_md::INLINE_LANGUAGE.into(),
			tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
			tree_sitter_md::INJECTION_QUERY_INLINE,
			"",
		),
		language!(
			"nix",
			tree_sitter_nix::language(),
			tree_sitter_nix::HIGHLIGHTS_QUERY,
			"",
			"",
		),
		language!(
			"ocaml",
			tree_sitter_ocaml::LANGUAGE_OCAML.into(),
			tree_sitter_ocaml::HIGHLIGHTS_QUERY,
			tree_sitter_ocaml::LOCALS_QUERY,
			"",
		),
		language!(
			"ocaml_interface",
			tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE.into(),
			tree_sitter_ocaml::HIGHLIGHTS_QUERY,
			tree_sitter_ocaml::LOCALS_QUERY,
			"",
		),
		language!(
			"python",
			tree_sitter_python::LANGUAGE.into(),
			tree_sitter_python::HIGHLIGHTS_QUERY,
			"",
			"",
		),
		language!(
			"regex",
			tree_sitter_regex::LANGUAGE.into(),
			tree_sitter_regex::HIGHLIGHTS_QUERY,
			"",
			"",
		),
		language!(
			"rust",
			tree_sitter_rust::LANGUAGE.into(),
			tree_sitter_rust::HIGHLIGHTS_QUERY,
			tree_sitter_rust::INJECTIONS_QUERY,
			"",
		),
		// language!(
		//     "scss",
		//     tree_sitter_scss::,
		//     merge![
		//         tree_sitter_css::HIGHLIGHTS_QUERY,
		//         tree_sitter_scss::HIGHLIGHTS_QUERY,
		//     ],
		//     "",
		//     "",
		// ),
		// language!(
		//     "query",
		//     tree_sitter_query::language(),
		//     tree_sitter_query::HIGHLIGHTS_QUERY,
		//     tree_sitter_query::INJECTIONS_QUERY,
		//     "",
		// ),
		// language!(
		//     "toml",
		//     tree_sitter_toml_ng::language(),
		//     tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
		//     "",
		//     "",
		// ),
		language!(
			"typescript",
			tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
			merge![
				query!("ecma/highlights"),
				tree_sitter_javascript::HIGHLIGHT_QUERY,
				tree_sitter_typescript::HIGHLIGHTS_QUERY,
			],
			tree_sitter_javascript::INJECTIONS_QUERY,
			merge![
				tree_sitter_javascript::LOCALS_QUERY,
				tree_sitter_typescript::LOCALS_QUERY,
			]
		),
		language!(
			"tsx",
			tree_sitter_typescript::LANGUAGE_TSX.into(),
			merge![
				query!("ecma/highlights"),
				tree_sitter_javascript::HIGHLIGHT_QUERY,
				tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
				tree_sitter_typescript::HIGHLIGHTS_QUERY,
			],
			tree_sitter_javascript::INJECTIONS_QUERY,
			merge![
				tree_sitter_javascript::LOCALS_QUERY,
				tree_sitter_typescript::LOCALS_QUERY,
			],
		),
	])
});

pub fn get_config(name: &str) -> Option<&'static HighlightConfiguration> {
	match EXTENSIONS.get(name) {
		Some(name) => CONFIGS.get(name),
		None => CONFIGS.get(name),
	}
}
