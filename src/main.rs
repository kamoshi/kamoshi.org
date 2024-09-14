mod html;
mod text;
mod ts;

use clap::{Parser, ValueEnum};
use hauchiwa::{Loader, Processor, Website};
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

	let processor_post = Processor {
		read_content: crate::html::post::parse_content,
		to_html: crate::html::post::as_html,
		to_link: crate::html::post::as_link,
	};

	let processor_slideshow = Processor {
		read_content: crate::html::slideshow::parse_content,
		to_html: crate::html::slideshow::as_html,
		to_link: crate::html::slideshow::as_link,
	};

	let processor_wiki = Processor {
		read_content: crate::html::wiki::parse_content,
		to_html: crate::html::wiki::as_html,
		to_link: crate::html::wiki::as_link,
	};

	let processor_flox = Processor {
		read_content: crate::html::parse_content,
		to_html: crate::html::as_html,
		to_link: crate::html::as_link,
	};

	let website = Website::design()
		.add_loaders(vec![
			Loader::glob_with::<Post>("content", "about.md", ["md"].into(), processor_post.clone()),
			Loader::glob_with::<Post>(
				"content",
				"posts/**/*",
				["md", "mdx"].into(),
				processor_post.clone(),
			),
			Loader::glob_with::<Slideshow>(
				"content",
				"slides/**/*",
				["md", "lhs"].into(),
				processor_slideshow,
			),
			Loader::glob_with::<Wiki>("content", "wiki/**/*", ["md"].into(), processor_wiki),
			Loader::glob_with::<Flox>("content", "projects/flox.md", ["md"].into(), processor_flox),
		])
		.js("search", "./js/search/dist/search.js")
		.js("photos", "./js/vanilla/photos.js")
		.js("reveal", "./js/vanilla/reveal.js")
		.js("editor", "./js/flox/main.ts")
		.js("lambda", "./js/flox/lambda.ts")
		.add_virtual(
			|sack| crate::html::map(sack).unwrap().render().to_owned().into(),
			"map/index.html".into(),
		)
		.add_virtual(|sack| crate::html::search(sack), "search/index.html".into())
		.add_virtual(
			|sack| {
				crate::html::to_list(
					sack,
					sack.get_links("projects/**/*.html"),
					"Projects".into(),
				)
			},
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
