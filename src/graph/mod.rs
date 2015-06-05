//! A simple graph data structure for storing the call graph we observe.

use std::collections::{HashMap, HashSet};
use std::io::{Result, Write};
use util::percent;

pub struct CallGraph {
    nodes: HashMap<String, NodeId>,
    node_counts: Vec<usize>,
    edges: HashMap<Edge, usize>,
    total: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct NodeId(usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Edge {
    caller: NodeId,
    callee: NodeId,
}

impl CallGraph {
    pub fn new() -> CallGraph {
        CallGraph { nodes: HashMap::new(), edges: HashMap::new(), node_counts: vec![], total: 0 }
    }

    pub fn node_id(&mut self, name: String) -> NodeId {
        let node_counts = &mut self.node_counts;
        *self.nodes.entry(name).or_insert_with(|| {
            let next_id = NodeId(node_counts.len());
            node_counts.push(0);
            next_id
        })
    }

    pub fn add_edges<I>(&mut self, frames: I)
        where I: Iterator<Item=String>
    {
        let mut node_ids: Vec<_> = frames.map(|frame| self.node_id(frame)).collect();
        let mut edges: Vec<_> =
            (1 .. node_ids.len())
            .map(|i| Edge { caller: node_ids[i-1], callee: node_ids[i] })
            .collect();

        node_ids.sort();
        node_ids.dedup();

        edges.sort();
        edges.dedup();

        for edge in edges {
            *self.edges.entry(edge).or_insert(0) += 1;
        }

        for id in node_ids {
            self.node_counts[id.0] += 1;
        }

        self.total += 1;
    }

    pub fn dump(&self, out: &mut Write, threshold: u32) -> Result<()> {
        try!(write!(out, "digraph G {{\n"));
        try!(write!(out, "  node [ shape=box ];"));

        let mut node_ids = HashSet::new();
        for (edge, &count) in self.edges.iter() {
            let percentage = percent(count, self.total);
            if percentage >= threshold {
                try!(write!(out, "  n{} -> n{} [label=\"{}%\"];\n",
                            edge.caller.0, edge.callee.0, percentage));
                node_ids.insert(edge.caller);
                node_ids.insert(edge.callee);
            }
        }

        for (name, &index) in self.nodes.iter() {
            let count = self.node_counts[index.0];
            let percentage = percent(count, self.total);
            if node_ids.contains(&index) {
                try!(write!(out, "  n{} [label=\"{} ({}%)\"];\n",
                            index.0, name, percentage));
            }
        }

        try!(write!(out, "}}\n"));
        Ok(())
    }
}
