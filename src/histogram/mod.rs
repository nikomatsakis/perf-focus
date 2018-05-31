//! A simple graph data structure for storing the call graph we observe.

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::Alignment;
use prettytable::format::consts::FORMAT_CLEAN;
use std::collections::HashMap;
use util::{percent, seconds_str};

use super::AddFrames;

pub struct Histogram {
    fns: HashMap<String, usize>,
}

impl Histogram {
    pub fn new() -> Histogram {
        Histogram {
            fns: HashMap::new(),
        }
    }

    pub fn dump(&self, total: usize, threshold: usize, frequency: usize) {
        let mut fns: Vec<(usize, &str)> = self.fns
            .iter()
            .map(|(key, &value)| (value, &key[..]))
            .collect();

        fns.sort();

        let mut table = Table::new();
        table.set_format(*FORMAT_CLEAN);

        let skip = if fns.len() < threshold {
            0
        } else {
            fns.len() - threshold
        };
        for &(count, name) in fns.iter().skip(skip) {
            let percentage = percent(count, total);
            table.add_row(Row::new(vec![
                Cell::new_align(&format!("{}%", percentage), Alignment::RIGHT),
                Cell::new_align(&seconds_str(count, frequency), Alignment::RIGHT),
                Cell::new(name),
            ]));
        }

        table.printstd();
    }

    pub fn get(&self, key: &str) -> usize {
        self.fns.get(key).cloned().unwrap_or(0)
    }
}

impl AddFrames for Histogram {
    fn add_frames<I>(&mut self, frames: I)
    where
        I: Iterator<Item = String>,
    {
        let mut frames: Vec<_> = frames.collect();
        frames.sort();
        frames.dedup();
        for frame in frames {
            *self.fns.entry(frame).or_insert(0) += 1;
        }
    }
}
