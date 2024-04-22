use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};

use crate::html::Linkable;
use crate::Sack;


pub enum AssetKind {
    Html(Box<dyn Fn(&Sack) -> String>),
    Image,
    Unknown,
    Bib(hayagriva::Library),
}

impl Debug for AssetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Html(ptr) => f.debug_tuple("Html").field(&"ptr").finish(),
            Self::Image => write!(f, "Image"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Bib(arg0) => f.debug_tuple("Bib").field(arg0).finish(),
        }
    }
}

#[derive(Debug)]
pub struct Asset {
    pub kind: AssetKind,
    pub out: Utf8PathBuf,
    pub link: Option<Linkable>,
    pub meta: super::Source,
}

pub struct Virtual(pub Utf8PathBuf, pub Box<dyn Fn(&Sack) -> String>);

impl Virtual {
    pub fn new<P, F>(path: P, call: F) -> Self
        where
            P: AsRef<Utf8Path>,
            F: Fn(&Sack) -> String + 'static
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

    let everything = Sack { assets: &assets };

    for item in items {
        match item {
            Item::Real(real) => render_real(real, &everything),
            Item::Fake(fake) => render_fake(fake, &everything),
        }
    }
}


fn render_real(item: &Asset, assets: &Sack) {
    match &item.kind {
        AssetKind::Html(render) => {
            let i = &item.meta.path;
            let o = Utf8Path::new("dist").join(&item.out);

            fs::create_dir_all(&o.parent().unwrap()).unwrap();

            let mut file = File::create(&o).unwrap();
            file.write_all(render(assets).as_bytes()).unwrap();

            println!("HTML: {} -> {}", i, o);
        },
        AssetKind::Image => {
            let i = &item.meta.path;
            let o = Utf8Path::new("dist").join(&item.out);
            fs::create_dir_all(&o.parent().unwrap()).unwrap();
            fs::copy(&i, &o).unwrap();
            println!("Image: {} -> {}", i, o);
        },
        AssetKind::Bib(_) => (),
        AssetKind::Unknown => {
            let i = &item.meta.path;
            let o = Utf8Path::new("dist").join(&item.out);
            fs::create_dir_all(&o.parent().unwrap()).unwrap();
            fs::copy(&i, &o).unwrap();
            println!("Unknown: {} -> {}", i, o);
        },
    }
}

fn render_fake(item: &Virtual, assets: &Sack) {
    let Virtual(out, render) = item;

    let o = Utf8Path::new("dist").join(&out);
    fs::create_dir_all(&o.parent().unwrap()).unwrap();

    let mut file = File::create(&o).unwrap();
    file.write_all(render(assets).as_bytes()).unwrap();
    println!("Virtual: -> {}", o);
}
