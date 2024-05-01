use std::fs::{self, File};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};

use crate::Sack;

use super::{Asset, AssetKind};


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
        .iter()
        .filter_map(|item| match item {
            Item::Real(a) => Some(a),
            Item::Fake(_) => None,
        })
        .collect();

    for item in items {
        match item {
            Item::Real(real) => render_real(real, &Sack::new(&assets, &real.out)),
            Item::Fake(fake) => render_fake(fake, &Sack::new(&assets, &fake.0)),
        }
    }
}


fn render_real(item: &Asset, sack: &Sack) {
    match &item.kind {
        AssetKind::Html(render) => {
            let i = &item.meta.src;
            let o = Utf8Path::new("dist").join(&item.out);

            fs::create_dir_all(o.parent().unwrap()).unwrap();

            let mut file = File::create(&o).unwrap();
            file.write_all(render(sack).as_bytes()).unwrap();

            println!("HTML: {} -> {}", i, o);
        },
        AssetKind::Image => {
            let i = &item.meta.src;
            let o = Utf8Path::new("dist").join(&item.out);
            fs::create_dir_all(o.parent().unwrap()).unwrap();
            fs::copy(i, &o).unwrap();
            println!("Image: {} -> {}", i, o);
        },
        AssetKind::Bib(_) => (),
        AssetKind::Other => {
            let i = &item.meta.src;
            let o = Utf8Path::new("dist").join(&item.out);
            fs::create_dir_all(o.parent().unwrap()).unwrap();
            fs::copy(i, &o).unwrap();
            println!("Unknown: {} -> {}", i, o);
        },
    }
}

fn render_fake(item: &Virtual, sack: &Sack) {
    let Virtual(out, render) = item;

    let o = Utf8Path::new("dist").join(out);
    fs::create_dir_all(o.parent().unwrap()).unwrap();

    let mut file = File::create(&o).unwrap();
    file.write_all(render(sack).as_bytes()).unwrap();
    println!("Virtual: -> {}", o);
}
