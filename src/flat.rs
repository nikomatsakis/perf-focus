use histogram::Histogram;
use std::collections::{HashMap, hash_map::Entry};
use std::iter;
use util::percent;

use super::AddFrames;

pub struct Flat {
    histogram: Histogram,
    leaves: HashMap<String, Leaf>,
    empty_traces: usize,
}

struct Leaf {
    contexts: HashMap<Vec<String>, usize>,
}

impl Flat {
    pub fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            leaves: HashMap::default(),
            empty_traces: 0,
        }
    }

    pub fn dump(&self, total_samples: usize) {
        let mut leaves: Vec<_> = self.leaves.iter()
            .map(|(leaf_name, leaf)| (leaf.percent(total_samples), leaf_name))
            .collect();

        leaves.sort();

        for &(leaf_percent, leaf_name) in leaves.iter().rev() {
            println!("{:3}% {}", leaf_percent, leaf_name);
        }

        if self.empty_traces > 0 {
            let percent = percent(self.empty_traces, total_samples);
            println!("{:3}% had no leaf function worth mentioning.", percent);
        }
    }

    fn insert(&mut self, mut trace: Vec<String>, count: usize) {
        let top = match trace.pop() {
            Some(p) => p,
            None => {
                self.empty_traces += count;
                return;
            }
        };

        match self.leaves.entry(top) {
            Entry::Vacant(slot) => {
                slot.insert(Leaf::new(trace, count));
            }

            Entry::Occupied(mut slot) => {
                slot.get_mut().insert_trace(trace, count);
            }
        }
    }

    pub fn rollup(&mut self, total_samples: usize, min_percent: usize) {
        let min_percent = min_percent as u32;

        loop {
            let leaves_to_remove: Vec<String> = self.leaves
                .keys()
                .filter_map(|leaf_name| {
                    let count = self.histogram.get(leaf_name);
                    let percent = percent(count, total_samples);
                    if percent < min_percent {
                        Some(leaf_name.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if leaves_to_remove.is_empty() {
                return;
            }

            for leaf_func in leaves_to_remove {
                let leaf = self.leaves.remove(&leaf_func).unwrap();
                leaf.distribute(self);
            }
        }
    }
}

impl Leaf {
    fn new(callers: Vec<String>, count: usize) -> Self {
        Leaf {
            contexts: iter::once((callers, count)).collect(),
        }
    }

    fn insert_trace(&mut self, context: Vec<String>, count: usize) {
        *self.contexts.entry(context).or_insert(0) += count;
    }

    fn count(&self) -> usize {
        self.contexts.values().sum()
    }

    fn percent(&self, total_samples: usize) -> u32 {
        percent(self.count(), total_samples)
    }

    fn distribute(self, flat: &mut Flat) {
        for (context, count) in self.contexts {
            flat.insert(context, count);
        }
    }
}

impl AddFrames for Flat {
    fn add_frames<I>(&mut self, frames: I)
    where
        I: Iterator<Item = String>,
    {
        let v: Vec<String> = frames.collect();
        self.insert(v.clone(), 1);
        self.histogram.add_frames(v.into_iter());
    }
}
