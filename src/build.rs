use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use camino::Utf8Path;

use crate::pipeline::{AssetKind, Output, OutputKind, Sack, Virtual};
use crate::BuildContext;

pub(crate) fn clean_dist() {
	println!("Cleaning dist");
	if fs::metadata("dist").is_ok() {
		fs::remove_dir_all("dist").unwrap();
	}
	fs::create_dir("dist").unwrap();
}

pub(crate) fn build_styles() {
	let css = grass::from_path("styles/styles.scss", &grass::Options::default()).unwrap();
	fs::write("dist/styles.css", css).unwrap();
}

pub(crate) fn build_content(ctx: &BuildContext, pending: &[&Output], hole: &[&Output]) {
	let now = std::time::Instant::now();
	render_all(ctx, pending, hole);
	println!("Elapsed: {:.2?}", now.elapsed());
}

pub(crate) fn build_static() {
	copy_recursively(std::path::Path::new("public"), std::path::Path::new("dist")).unwrap();
}

pub(crate) fn build_pagefind() {
	let res = Command::new("pagefind")
		.args(["--site", "dist"])
		.output()
		.unwrap();

	println!("{}", String::from_utf8(res.stdout).unwrap());
}

pub(crate) fn build_js() {
	let res = Command::new("esbuild")
		.arg("js/vanilla/reveal.js")
		.arg("js/vanilla/photos.ts")
		.arg("js/search/dist/search.js")
		.arg("--format=esm")
		.arg("--bundle")
		.arg("--splitting")
		.arg("--minify")
		.arg("--outdir=dist/js/")
		.output()
		.unwrap();

	println!("{}", String::from_utf8(res.stderr).unwrap());
}

fn copy_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
	fs::create_dir_all(&dst)?;
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let filetype = entry.file_type()?;
		if filetype.is_dir() {
			copy_recursively(entry.path(), dst.as_ref().join(entry.file_name()))?;
		} else {
			fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}

fn render_all(ctx: &BuildContext, pending: &[&Output], hole: &[&Output]) {
	for item in pending {
		let file = match &item.kind {
			OutputKind::Asset(a) => Some(&a.meta.path),
			OutputKind::Virtual(_) => None,
		};
		render(
			item,
			Sack {
				ctx,
				hole,
				path: &item.path,
				file,
			},
		);
	}
}

fn render(item: &Output, sack: Sack) {
	let o = Utf8Path::new("dist").join(&item.path);
	fs::create_dir_all(o.parent().unwrap()).unwrap();

	match item.kind {
		OutputKind::Asset(ref real) => {
			let i = &real.meta.path;

			match &real.kind {
				AssetKind::Html(closure) => {
					let mut file = File::create(&o).unwrap();
					file.write_all(closure(&sack).as_bytes()).unwrap();
					println!("HTML: {} -> {}", i, o);
				}
				AssetKind::Bibtex(_) => {}
				AssetKind::Image => {
					fs::create_dir_all(o.parent().unwrap()).unwrap();
					fs::copy(i, &o).unwrap();
					println!("Image: {} -> {}", i, o);
				}
			};
		}
		OutputKind::Virtual(Virtual(ref closure)) => {
			let mut file = File::create(&o).unwrap();
			file.write_all(closure(&sack).as_bytes()).unwrap();
			println!("Virtual: -> {}", o);
		}
	}
}
