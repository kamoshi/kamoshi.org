mod html;
mod text;
mod ts;

use clap::{Parser, ValueEnum};
use hauchiwa::Website;
use html::{Flox, Post, Slideshow, Wiki};
use hypertext::Renderable;

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

	let website = Website::design()
		.content::<Post>("content/about.md", ["md"].into())
		.content::<Post>("content/posts/**/*", ["md", "mdx"].into())
		.content::<Slideshow>("content/slides/**/*", ["md", "lhs"].into())
		.content::<Wiki>("content/wiki/**/*", ["md"].into())
        .content::<Flox>("content/projects/flox.md", ["md"].into())
		.js("search", "./js/search/dist/search.js")
		.js("photos", "./js/vanilla/photos.js")
		.js("reveal", "./js/vanilla/reveal.js")
		.js("editor", "./js/flox/main.ts")
		.add_virtual(
			|sack| crate::html::map(sack).render().to_owned().into(),
			"map/index.html".into(),
		)
		.add_virtual(
			|sack| crate::html::search(sack).render().to_owned().into(),
			"search/index.html".into(),
		)
		.add_virtual(
			|sack| crate::html::to_list(sack, sack.get_links("projects/**/*.html"), "Projects".into()),
			"projects/index.html".into(),
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
				let (parsed, _, _) = text::md::parse(&data, sack, "".into(), None);
				crate::html::home(sack, &parsed)
			},
			"index.html".into(),
		)
		.finish();

	match args.mode {
		Mode::Build => website.build(),
		Mode::Watch => website.watch(),
	}
}
