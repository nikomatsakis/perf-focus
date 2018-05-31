use ::tree::Tree;
use ::util::percent;

use super::AddFrames;

pub struct LeafList {
    tree: Tree
}

impl LeafList {
    pub fn prepare(&mut self, total_samples: usize, max_depth: usize, min_percent: usize) {
        self.tree.sort();
        self.tree.rollup(total_samples, max_depth, min_percent);
    }

    pub fn dump(&self, total_samples: usize, max_depth: usize, min_percent: usize) {
        let mut leaves = vec![];
        self.tree.for_each_leaf(|label, hits| {
            leaves.push((hits, label));
        };);
        leaves.sort();
        for (hits, label) in leaves {
            let percent = percent(hits, total_samples);
            print!("{} ({}%)", 
        });
    }
}

impl AddFrames for Tree {
    fn add_frames<I>(&mut self, frames: I)
        where I: Iterator<Item=String>
    {
        self.root_node.add_frames(frames);
    }
}
