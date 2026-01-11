//// datalog.rs
use std::collections::HashMap;

// rules
crepe::crepe! {
    @input
    struct Link(usize, usize);

    @output
    struct Backlink(usize, usize);

    Backlink(target, source) <- Link(source, target);
}

pub struct Datalog {
    runtime: Crepe,
    // Maps external string keys to internal usize IDs
    interner: HashMap<String, usize>,
    // Maps internal usize IDs back to external string keys
    reverser: Vec<String>,
}

impl Datalog {
    pub fn new() -> Self {
        Self {
            runtime: Crepe::new(),
            interner: HashMap::new(),
            reverser: Vec::new(),
        }
    }

    /// Helper: Get or create a unique ID for a given key
    fn intern(&mut self, key: &str) -> usize {
        if let Some(&id) = self.interner.get(key) {
            id
        } else {
            let id = self.reverser.len();
            self.reverser.push(key.to_string());
            self.interner.insert(key.to_string(), id);
            id
        }
    }

    /// Add a link using string keys
    pub fn add_link(&mut self, source: &str, target: &str) {
        let source_id = self.intern(source);
        let target_id = self.intern(target);
        self.runtime.link.push(Link(source_id, target_id));
    }

    pub fn solve(self) -> Solution {
        let (backlinks,) = self.runtime.run();

        Solution {
            // We move the reverse map into the solution so we can look up names later
            names: self.reverser,
            backlinks: {
                let mut map = HashMap::new();
                for Backlink(target, source) in backlinks {
                    map.entry(target).or_insert_with(Vec::new).push(source);
                }
                map
            },
        }
    }
}

pub struct Solution {
    // Stores the mapping from ID -> String
    names: Vec<String>,
    // Stores ID -> List<ID>
    backlinks: HashMap<usize, Vec<usize>>,
}

impl Solution {
    /// Returns the list of source `hrefs` that link to the target `href`
    pub fn get_backlinks(&self, target_href: &str) -> Option<Vec<&str>> {
        let target_id = self.names.iter().position(|n| n == target_href)?;

        self.backlinks.get(&target_id).map(|sources| {
            sources
                .iter()
                .map(|&source_id| self.names[source_id].as_str())
                .collect()
        })
    }
}
