//! A simple graph data structure for storing the call graph we observe.

use std::collections::{HashMap};
use util::percent;

use super::AddFrames;

pub struct Histogram {
    fns: HashMap<String, usize>,
}

impl Histogram {
    pub fn new() -> Histogram {
        Histogram { fns: HashMap::new(), }
    }

    pub fn dump(&self, total: usize, threshold: usize) {
        let mut fns: Vec<(usize, &str)> =
            self.fns.iter()
                    .map(|(key, &value)| (value, &key[..]))
                    .collect();

        fns.sort();

        let skip = if fns.len() < threshold {0} else {fns.len() - threshold};
        for &(count, name) in fns.iter().skip(skip) {
            let percentage = percent(count, total);
            println!("{:3}% {}", percentage, name);
        }
    }
}

impl AddFrames for Histogram {
    fn add_frames<I>(&mut self, frames: I)
        where I: Iterator<Item=String>
    {
        let mut frames: Vec<_> = frames.collect();
        frames.sort();
        frames.dedup();
        for frame in frames {
            *self.fns.entry(frame).or_insert(0) += 1;
        }
    }
}
