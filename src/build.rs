use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};

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
) -> HashMap<Utf8PathBuf, Utf8PathBuf> {
	let now = std::time::Instant::now();
	let hashes = render_all(ctx, pending, hole, hash);
	println!("Elapsed: {:.2?}", now.elapsed());
	copy_recursively(Path::new(".hash"), Path::new("dist/hash")).unwrap();
	let mut lmao = HashMap::<Utf8PathBuf, Utf8PathBuf>::new();
	for hash in hashes {
		lmao.insert(hash.file, hash.hash);
	}
	lmao
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
) -> Vec<Hashed> {
	pending
		.iter()
		.filter_map(|item| {
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

fn store_hash_png(data: &[u8]) -> Utf8PathBuf {
	let store = Utf8Path::new(".hash");
	let ident = sha256::digest(data);
	let store_hash = store.join(&ident).with_extension("webp");

	if !store_hash.exists() {
		let img = image::load_from_memory(data).expect("Couldn't load image");
		let dim = (img.width(), img.height());
		let mut out = Vec::new();
		let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut out);
		encoder.encode(&img.to_rgba8(), dim.0, dim.1, image::ColorType::Rgba8).expect("Encoding error");

		fs::create_dir_all(store).unwrap();
		fs::write(store_hash, out).expect("Couldn't output optimized image");
	}

	Utf8Path::new("/")
		.join("hash")
		.join(ident)
		.with_extension("webp")
}

#[derive(Debug)]
pub(crate) struct Hashed {
	pub file: Utf8PathBuf,
	pub hash: Utf8PathBuf,
}

fn render(item: &Output, sack: Sack) -> Option<Hashed> {
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
					None
				}
				AssetKind::Bibtex(_) => None,
				AssetKind::Image => {
					let hash = match item.path.extension() {
						Some("png") => Some(store_hash_png(&std::fs::read(i).unwrap())),
						Some("") => None,
						_ => None,
					};

					fs::create_dir_all(o.parent().unwrap()).unwrap();
					fs::copy(i, &o).unwrap();
					println!("Image: {} -> {}", i, o);

					hash.map(|hash| Hashed {
						file: item.path.to_owned(),
						hash,
					})
				}
			}
		}
		OutputKind::Virtual(Virtual(ref closure)) => {
			let mut file = File::create(&o).unwrap();
			file.write_all(closure(&sack).as_bytes()).unwrap();
			println!("Virtual: -> {}", o);
			None
		}
	}
}
