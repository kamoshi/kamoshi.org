use std::collections::HashMap;

// rules
crepe::crepe! {

    // links

    @input
    struct Link(usize, usize);

    @output
    struct Backlink(usize, usize);

    Backlink(target, source) <- Link(source, target);

    // hierarchy

    @input
    struct Parent(usize, usize);

    @output
    struct Child(usize, usize);

    Child(child, parent) <- Parent(parent, child);

    // co-citation

    @output
    struct CoCitation(usize, usize, usize);

    CoCitation(witness, a, b) <- Link(witness, a), Link(witness, b), (a < b);
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

    pub fn add_parent(&mut self, parent: &str, child: &str) {
        let p_id = self.intern(parent);
        let c_id = self.intern(child);
        self.runtime.parent.push(Parent(p_id, c_id));
    }

    pub fn solve(self) -> Solution {
        let parents = {
            let mut map = HashMap::new();
            for &Parent(parent, child) in &self.runtime.parent {
                map.insert(child, parent);
            }
            map
        };

        let (backlinks, children, co_citations) = self.runtime.run();

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
            children: {
                let mut map = HashMap::new();
                for Child(child, parent) in children {
                    map.entry(parent).or_insert_with(Vec::new).push(child);
                }
                map
            },
            parents,
            co_citations: {
                let mut map = HashMap::new();
                for CoCitation(_, a, b) in co_citations {
                    *map.entry(a)
                        .or_insert_with(HashMap::new)
                        .entry(b)
                        .or_default() += 1;
                    *map.entry(b)
                        .or_insert_with(HashMap::new)
                        .entry(a)
                        .or_default() += 1;
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
    // Map child_id -> parent_id
    parents: HashMap<usize, usize>,
    // Map parent_id -> Vec<child_id>
    children: HashMap<usize, Vec<usize>>,
    // Map (parent_id, child_id) -> count
    co_citations: HashMap<usize, HashMap<usize, usize>>,
}

impl Solution {
    pub fn get_backlinks(&self, target_href: &str) -> Option<Vec<&str>> {
        let target_id = self.names.iter().position(|n| n == target_href)?;
        self.backlinks
            .get(&target_id)
            .map(|sources| sources.iter().map(|&id| self.names[id].as_str()).collect())
    }

    /// Get the parent href for a given child href
    pub fn get_parent(&self, child_href: &str) -> Option<&str> {
        let child_id = self.names.iter().position(|n| n == child_href)?;
        self.parents
            .get(&child_id)
            .map(|&id| self.names[id].as_str())
    }

    /// Get children hrefs for a given parent href
    pub fn get_children(&self, parent_href: &str) -> Option<Vec<&str>> {
        let parent_id = self.names.iter().position(|n| n == parent_href)?;
        self.children
            .get(&parent_id)
            .map(|children| children.iter().map(|&id| self.names[id].as_str()).collect())
    }

    /// Get co-citations for a given parent href
    pub fn get_co_citations(&self, href: &str) -> Option<Vec<(&str, usize)>> {
        let id = self.names.iter().position(|n| n == href)?;
        self.co_citations.get(&id).map(|co_citations| {
            co_citations
                .iter()
                .map(|(&id, &count)| (self.names[id].as_str(), count))
                .collect()
        })
    }
}
