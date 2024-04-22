use std::collections::HashSet ;

use camino::Utf8PathBuf;
use glob::glob;


#[derive(Debug)]
pub enum SourceKind {
    Index,
    Asset,
}

#[derive(Debug)]
pub struct Source {
    pub kind: SourceKind,
    pub ext: String,
    pub dirs: Utf8PathBuf,
    pub path: Utf8PathBuf,
}


fn to_source(path: Utf8PathBuf, exts: &HashSet<&'static str>) -> Source {
    let dir = path.parent().unwrap();
    let ext = path.extension().unwrap();

    if !exts.contains(ext) {
        return Source {
            kind: SourceKind::Asset,
            ext: ext.to_owned(),
            dirs: dir.to_owned(),
            path,
        };
    }

    let dirs = match path.file_stem().unwrap() {
        "index" => dir.to_owned(),
        name    => dir.join(name),
    };

    Source {
        kind: SourceKind::Index,
        ext: ext.to_owned(),
        dirs,
        path,
    }
}


pub fn gather(pattern: &str, exts: &HashSet<&'static str>) -> Vec<Source> {
    glob(pattern)
        .unwrap()
        .into_iter()
        .filter_map(|path| {
            let path = path.unwrap();
            let path = Utf8PathBuf::from_path_buf(path).expect("Filename is not valid UTF8");

            match path.is_dir() {
                true  => None,
                false => Some(to_source(path, exts))
            }
        })
        .collect()
}
