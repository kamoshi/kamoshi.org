use std::collections::HashMap;

// rules
crepe::crepe! {
    @input
    struct Link(usize, usize);

    @output
    struct Backlink(usize, usize);

    // If Source -> Target, then Target has Backlink from Source
    Backlink(target, source) <- Link(source, target);
}

pub struct Datalog {
    // Datalog runtime
    runtime: Crepe,
}

impl Datalog {
    pub fn new() -> Self {
        Self {
            runtime: Crepe::new(),
        }
    }

    pub fn add_link(&mut self, source: usize, target: usize) {
        self.runtime.link.push(Link(source, target));
    }

    /// Run the solver and return a map of Target -> List of Sources
    pub fn solve(self) -> Solution {
        let (backlinks,) = self.runtime.run();

        Solution {
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
    backlinks: HashMap<usize, Vec<usize>>,
}

impl Solution {
    pub fn get_backlinks(&self, target: usize) -> Option<&[usize]> {
        self.backlinks.get(&target).map(Vec::as_slice)
    }
}
