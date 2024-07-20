mod html;
mod text;
mod ts;

use clap::{Parser, ValueEnum};
use hauchiwa::{process_content, Website};
use hypertext::{Raw, Renderable};

#[derive(Parser, Debug, Clone)]
struct Args {
	#[clap(value_enum, index = 1, default_value = "build")]
	mode: Mode,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Mode {
	Build,
	Watch,
}

fn main() {
	let args = Args::parse();

	let website = Website::new()
		.add_source(
			"content/about.md",
			["md"].into(),
			process_content::<crate::html::Post>,
		)
		.add_source(
			"content/posts/**/*",
			["md", "mdx"].into(),
			process_content::<crate::html::Post>,
		)
		.add_source(
			"content/slides/**/*",
			["md", "lhs"].into(),
			process_content::<crate::html::Slideshow>,
		)
		.add_source(
			"content/wiki/**/*",
			["md"].into(),
			process_content::<crate::html::Wiki>,
		)
		.add_virtual(
			|sack| crate::html::map(sack).render().to_owned().into(),
			"map/index.html".into(),
		)
		.add_virtual(
			|sack| crate::html::search(sack).render().to_owned().into(),
			"search/index.html".into(),
		)
		.add_virtual(
			|sack| crate::html::to_list(sack, sack.get_links("posts/**/*.html"), "Posts".into()),
			"posts/index.html".into(),
		)
		.add_virtual(
			|sack| {
				crate::html::to_list(
					sack,
					sack.get_links("slides/**/*.html"),
					"Slideshows".into(),
				)
			},
			"slides/index.html".into(),
		)
		.add_virtual(
			|sack| {
				let data = std::fs::read_to_string("content/index.md").unwrap();
				let (_, html, _) = text::md::parse(
					data,
					None,
					"".into(),
					sack.hash
						.as_ref()
						.map(ToOwned::to_owned)
						.unwrap_or_default(),
				);
				crate::html::home(sack, Raw(html))
					.render()
					.to_owned()
					.into()
			},
			"index.html".into(),
		)
		.finish();

	match args.mode {
		Mode::Build => website.build(),
		Mode::Watch => website.watch(),
	}
}
