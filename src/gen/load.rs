use std::collections::HashSet ;
use std::path::PathBuf;

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
    pub dirs: PathBuf,
    pub path: PathBuf,
}


fn to_source(path: PathBuf, exts: &HashSet<&'static str>) -> Source {
    let dirs = path.parent().unwrap();
    let ext = path.extension().unwrap().to_str().unwrap();

    if !exts.contains(ext) {
        return Source {
            kind: SourceKind::Asset,
            ext: ext.to_owned(),
            dirs: dirs.to_owned(),
            path,
        };
    }

    let dirs = match path.file_stem().unwrap().to_str().unwrap() {
        "index" => dirs.to_owned(),
        name    => dirs.join(name),
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

            match path.is_dir() {
                true  => None,
                false => Some(to_source(path, exts))
            }
        })
        .collect()
}
