use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

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

pub(crate) fn build_content(
	ctx: &BuildContext,
	pending: &[&Output],
	hole: &[&Output],
	hash: Option<HashMap<Utf8PathBuf, Utf8PathBuf>>,
) {
	let now = std::time::Instant::now();
	render_all(ctx, pending, hole, hash);
	println!("Elapsed: {:.2?}", now.elapsed());
	copy_recursively(Path::new(".hash"), Path::new("dist/hash")).unwrap();
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

fn render_all(
	ctx: &BuildContext,
	pending: &[&Output],
	hole: &[&Output],
	hash: Option<HashMap<Utf8PathBuf, Utf8PathBuf>>,
) {
	pending
		.iter()
		.map(|item| {
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
					hash: hash.clone(),
				},
			)
		})
		.collect()
}

fn store_hash(buffer: &[u8]) -> Utf8PathBuf {
	let store = Utf8Path::new(".hash");
	let hash = sha256::digest(buffer);
	let img = image::load_from_memory(buffer).expect("Couldn't load image");
	let store_hash = store.join(&hash).with_extension("webp");

	if !store_hash.exists() {
		let dim = (img.width(), img.height());
		let mut out = Vec::new();
		let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut out);

		encoder
			.encode(&img.to_rgba8(), dim.0, dim.1, image::ColorType::Rgba8)
			.expect("Encoding error");

		fs::create_dir_all(store).unwrap();
		fs::write(store_hash, out).expect("Couldn't output optimized image");
	}

	Utf8Path::new("/")
		.join("hash")
		.join(hash)
		.with_extension("webp")
}

pub(crate) fn store_hash_all(items: &[&Output]) -> Vec<Hashed> {
	items
		.par_iter()
		.filter_map(|item| match item.kind {
			OutputKind::Asset(ref asset) => match asset.kind {
				AssetKind::Image => {
					let buffer = std::fs::read(&asset.meta.path).expect("Couldn't read file");
					let format = image::guess_format(&buffer).expect("Couldn't read format");

					if matches!(format, image::ImageFormat::Gif) {
						return None;
					}

					let hash = store_hash(&buffer);
					println!("Hashing image {} as {}", asset.meta.path, hash);

					Some(Hashed {
						file: item.path.to_owned(),
						hash,
					})
				}
				_ => None,
			},
			_ => None,
		})
		.collect()
}

#[derive(Debug)]
pub(crate) struct Hashed {
	pub file: Utf8PathBuf,
	pub hash: Utf8PathBuf,
}

fn render(item: &Output, sack: Sack) {
	let dist = Utf8Path::new("dist");
	let o = dist.join(&item.path);
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
				AssetKind::Bibtex(_) => (),
				AssetKind::Image => {
					fs::create_dir_all(o.parent().unwrap()).unwrap();
					fs::copy(i, &o).unwrap();
					println!("Image: {} -> {}", i, o);
				}
			}
		}
		OutputKind::Virtual(Virtual(ref closure)) => {
			let mut file = File::create(&o).unwrap();
			file.write_all(closure(&sack).as_bytes()).unwrap();
			println!("Virtual: -> {}", o);
		}
	}
}
