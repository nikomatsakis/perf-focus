//! Builds up a tree of callers or callees.
//!

/*

Suppose we have the following samples:

- A
  - B1
    - C

- A
  - B2
    - C

- A
  - B1
    - D

We want to arrange into a tree:

- A (3/3, 0/3 self)
  - B1 (2/3, 0/3 self)
    - C (1/3, 1/3 self)
    - D (1/3, 1/3 self)
  - B2
    - D (1/3, 1/3 self)

*/

use util::percent;

use super::AddFrames;

pub struct Tree {
    root_node: TreeNode,
}

pub struct TreeNode {
    /// label on the node
    label: String,

    /// number of samples that passed through this node
    hits_total: usize,

    /// number of samples that terminated on this node
    hits_self: usize,

    /// things invoked by us
    children: Vec<TreeNode>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            root_node: TreeNode::new("<root>".to_string()),
        }
    }

    pub fn sort(&mut self) {
        self.root_node.sort();
    }

    pub fn dump(&self, total_samples: usize, max_depth: usize, min_percent: usize) {
        for child in &self.root_node.children {
            child.dump(
                0,
                total_samples,
                max_depth,
                min_percent,
            );
        }
    }
}

impl TreeNode {
    fn new(label: String) -> TreeNode {
        TreeNode {
            label: label,
            hits_total: 0,
            hits_self: 0,
            children: vec![],
        }
    }

    fn sort(&mut self) {
        self.children.sort_by_key(|c| ::std::usize::MAX - c.hits_total);
        for c in &mut self.children {
            c.sort();
        }
    }

    fn dump(
        &self,
        parents: usize,
        total_samples: usize,
        max_depth: usize,
        min_percent: usize,
    ) {
        let self_percent = percent(self.hits_self, total_samples);
        let total_percent = percent(self.hits_total, total_samples);

        if (total_percent as usize) < min_percent {
            return;
        }

        for _ in 0 .. parents {
            print!(": ");
        }

        print!("| {} ({}% total, {}% self)", self.label, total_percent, self_percent);

        if !self.children.is_empty() && (parents + 1 > max_depth) {
            println!(" [...]");
            return;
        }

        println!();
        for c in &self.children {
            c.dump(parents + 1, total_samples, max_depth, min_percent);
        }
    }

    fn add_frames<I>(&mut self, mut frames: I)
        where I: Iterator<Item=String>
    {
        self.hits_total += 1;

        if let Some(child_label) = frames.next() {
            for child_node in &mut self.children {
                if child_node.label == child_label {
                    return child_node.add_frames(frames);
                }
            }

            self.children.push(TreeNode::new(child_label.to_string()));
            self.children.last_mut()
                         .unwrap()
                         .add_frames(frames);
        } else {
            self.hits_self += 1;
        }
    }
}

impl AddFrames for Tree {
    fn add_frames<I>(&mut self, frames: I)
        where I: Iterator<Item=String>
    {
        self.root_node.add_frames(frames);
    }
}
