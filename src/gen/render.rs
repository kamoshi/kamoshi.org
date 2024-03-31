use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::Write;

use crate::html::LinkableData;


pub enum AssetKind {
    Html(Box<dyn Fn(&[&Asset]) -> String>),
    Image,
    Unknown,
    Bib(hayagriva::Library),
}

pub struct Asset {
    pub kind: AssetKind,
    pub out: PathBuf,
    pub link: Option<LinkableData>,
    pub meta: super::Source,
}

pub struct Virtual(pub PathBuf, pub Box<dyn Fn(&[&Asset]) -> String>);

impl Virtual {
    pub fn new<P, F>(path: P, call: F) -> Self
        where
            P: AsRef<Path>,
            F: Fn(&[&Asset]) -> String + 'static
    {
        Self(path.as_ref().into(), Box::new(call))
    }
}

pub enum Item {
    Real(Asset),
    Fake(Virtual),
}

impl From<Asset> for Item {
    fn from(value: Asset) -> Self {
        Item::Real(value)
    }
}

impl From<Virtual> for Item {
    fn from(value: Virtual) -> Self {
        Item::Fake(value)
    }
}


pub fn render(items: &[Item]) {
    let assets: Vec<&Asset> = items
        .into_iter()
        .filter_map(|item| match item {
            Item::Real(a) => Some(a),
            Item::Fake(_) => None,
        })
        .collect();

    for item in items {
        match item {
            Item::Real(real) => render_real(real, &assets),
            Item::Fake(fake) => render_fake(fake, &assets),
        }
    }
}


fn render_real(item: &Asset, assets: &[&Asset]) {
    match &item.kind {
        AssetKind::Html(render) => {
            let i = &item.meta.path;
            let o = Path::new("dist").join(&item.out);

            fs::create_dir_all(&o.parent().unwrap()).unwrap();

            let mut file = File::create(&o).unwrap();
            file.write_all(render(assets).as_bytes()).unwrap();

            println!("HTML: {} -> {}", i.to_str().unwrap(), o.to_str().unwrap());
        },
        AssetKind::Image => {
            let i = &item.meta.path;
            let o = Path::new("dist").join(&item.out);
            fs::create_dir_all(&o.parent().unwrap()).unwrap();
            fs::copy(&i, &o).unwrap();
            println!("Image: {} -> {}", i.to_str().unwrap(), o.to_str().unwrap());
        },
        AssetKind::Bib(_) => (),
        AssetKind::Unknown => {
            let i = &item.meta.path;
            let o = Path::new("dist").join(&item.out);
            fs::create_dir_all(&o.parent().unwrap()).unwrap();
            fs::copy(&i, &o).unwrap();
            println!("Unknown: {} -> {}", i.to_str().unwrap(), o.to_str().unwrap());
        },
    }
}

fn render_fake(item: &Virtual, assets: &[&Asset]) {
    let Virtual(out, render) = item;

    let o = Path::new("dist").join(&out);
    fs::create_dir_all(&o.parent().unwrap()).unwrap();

    let mut file = File::create(&o).unwrap();
    file.write_all(render(assets).as_bytes()).unwrap();
    println!("Virtual: -> {}", o.to_str().unwrap());
}
