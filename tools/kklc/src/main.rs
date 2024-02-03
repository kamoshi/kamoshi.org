use std::{collections::HashMap, error::Error};

use std::fs::File;
use serde::{Serialize, Deserialize};
use csv::{ReaderBuilder, StringRecord};


const PATH_CSV: &str = "kklc.csv";


type Readings = HashMap<String, Vec<String>>;
type Ruby = Vec<(String, Option<String>)>;

#[derive(Debug, Serialize, Deserialize)]
struct KKLCEntry {
    id: i32,
    char: String,
    keys: Vec<String>,
    senses: Vec<String>,
    onyomi: Vec<String>,
    kunyomi: Vec<String>,
    examples: Vec<(String, Ruby)>,
}


fn get_readings() -> HashMap<String, Vec<String>> {
    ReaderBuilder::new()
        .from_path(PATH_CSV)
        .expect("Failed opening CSV")
        .records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");

            let kanji = record.get(1).unwrap();
            let onyomi = record.get(3)
                .unwrap()
                .split("\n\n")
                .map(|s| s.split('(').next().unwrap().to_owned());
            let kunyomi = record.get(4)
                .unwrap()
                .split("\n\n")
                .map(|s| s.split('(').next().unwrap().to_owned());

            (kanji.to_owned(), kunyomi.chain(onyomi).collect())
        })
        .collect()
}

fn map_vocab(str: &str) -> (String, String, String) {
    let mut iter = str.split(" ・ ").map(String::from);
    (
        iter.next().unwrap()
            .replace("¹", "")
            .replace("²", "")
            .replace("…", "")
            .replace("*", "")
            .trim()
            .into(),
        iter.next().unwrap()
            .split('[').next().unwrap()
            .split('(').next().unwrap()
            .replace(" ", "")
            .replace("¹", "")
            .replace("²", "")
            .replace("…", "")
            .trim()
            .into(),
        iter.next().unwrap(),
    )
}

fn map_ruby(fst: &str, snd: &str, readings: &Readings) -> Ruby {
    if !fst.contains("○") {
        let mapping = furigana::map(fst, snd, &readings)
            .into_iter()
            .max_by_key(|f| f.accuracy);

        if let Some(mapping) = mapping {
            if mapping.accuracy > 0 {
                return mapping.furigana
                    .into_iter()
                    .map(|x| (x.segment.to_owned(), x.furigana.map(String::from)))
                    .collect()
            }
        }
    }

    vec![(fst.to_owned(), Some(snd.to_owned()))]
}

fn map_example(vocab: &[(String, String, String)], readings: &Readings) -> Vec<(String, Ruby)> {
    vocab.iter()
        .map(|(fst, snd, word)| (word.to_owned(), map_ruby(fst, snd, readings)))
        .collect()
}

fn map_record(record: StringRecord, readings: &Readings) -> KKLCEntry {
    let id = record.get(0)
        .unwrap()
        .parse()
        .unwrap();
    let char: String = record.get(1)
        .unwrap()
        .into();
    let keys = record.get(2)
        .unwrap_or("")
        .split("\n\n")
        .map(String::from)
        .collect();
    let onyomi = record.get(3)
        .unwrap_or("")
        .split("\n\n")
        .map(String::from)
        .collect::<Vec<_>>();
    let kunyomi = record.get(4)
        .unwrap_or("")
        .split("\n\n")
        .map(String::from)
        .collect::<Vec<_>>();
    let vocab = record.get(5)
        .unwrap_or("")
        .split("\n\n")
        .map(map_vocab)
        .collect::<Vec<_>>();
    let senses = record.get(6)
        .unwrap_or("")
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let examples = map_example(&vocab, readings);

    KKLCEntry { id, char, keys, senses, onyomi, kunyomi, examples }
}

fn get_entries(readings: &HashMap<String, Vec<String>>) -> Vec<KKLCEntry> {
    let mut reader = ReaderBuilder::new()
        .from_path(PATH_CSV)
        .expect("Error opening file");

    reader.records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");
            map_record(record, readings)
        })
        .collect()
}


fn main() -> Result<(), Box<dyn Error>> {
    let readings = get_readings();
    let entries = get_entries(&readings);

    for (i, entry) in entries.iter().enumerate() {
        let path = format!("../../public/static/kanji/{}.json", i + 1);
        let file = File::create(path).expect("Error creating file");

        serde_json::to_writer(&file, entry).expect("Error writing JSON to file");
    }

    Ok(())
}
