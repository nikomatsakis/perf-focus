//! A simple graph data structure for storing the call graph we observe.

use std::collections::{HashMap, HashSet};
use std::io::{Result, Write};
use std::usize;
use util::percent;

use super::AddFrames;

pub struct CallGraph {
    nodes: HashMap<String, NodeId>,
    edges: HashMap<Edge, usize>,
    node_counts: Vec<usize>,

    // Each frame of data is inlined into this
    // vector, separated by by MARKER values.
    // So if we are given 22, 23, 24 and 25, 26, 27
    // as two frames, this vector would be
    // `22, 23, 24, MARKER, 25, 26, 27, MARKER`.
    //
    // We never allow empty samples, so you shouldn't see MARKER,
    // MARKER.
    //
    // The intention is to lower memory overhead versus
    // Vec<Vec<NodeId>>, but I'm not sure how clever this really is.
    frames: Vec<NodeId>,

    total: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct NodeId(usize);

const MARKER: NodeId = NodeId(usize::MAX);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Edge {
    caller: NodeId,
    callee: NodeId,
}

impl CallGraph {
    pub fn new() -> CallGraph {
        CallGraph { nodes: HashMap::new(), edges: HashMap::new(), node_counts: vec![], total: 0,
                    frames: vec![] }
    }

    pub fn set_total(&mut self, total: usize, threshold: usize) {
        self.total = total;

        let mut percents: Vec<(u32, NodeId)> =
            self.node_counts.iter()
                            .map(|&count| percent(count, total))
                            .enumerate()
                            .map(|(index, percent)| (percent, NodeId(index)))
                            .collect();

        percents.sort();

        // a map of the top N node ids, and their percentages
        let top_node_ids: HashSet<NodeId> =
            percents.iter()
                    .rev()
                    .take(threshold)
                    .map(|&(_, i)| i)
                    .collect();

        // drop all nodes from the list of frames data if they don't
        // appear in the top-node-ids list
        self.frames.retain(|&n| n == MARKER || top_node_ids.contains(&n));

        // construct the edges.
        let mut edges = vec![];
        for i in 0..self.frames.len() {
            let caller = self.frames[i];

            // when we reach the end of a sample, collect the edges,
            // remove duplicates, and insert them into the map. This
            // way, if an edge occurs multiple times within one
            // sample, it only gets counted a single time in the map.
            if caller == MARKER {
                edges.sort();
                edges.dedup();
                for &edge in &edges {
                    *self.edges.entry(edge).or_insert(0) += 1;
                }
                edges.truncate(0);
                continue;
            }

            // otherwise, record an edge between this frame and the next
            let callee = self.frames[i+1];
            if callee != MARKER {
                edges.push(Edge { caller: caller, callee: callee });
            }
        }
    }

    pub fn node_id(&mut self, name: String) -> NodeId {
        let node_counts = &mut self.node_counts;
        *self.nodes.entry(name).or_insert_with(|| {
            let next_id = NodeId(node_counts.len());
            node_counts.push(0);
            next_id
        })
    }

    pub fn dump(&self, out: &mut Write) -> Result<()> {
        try!(write!(out, "digraph G {{\n"));
        try!(write!(out, "  node [ shape=box ];"));

        let mut node_ids = HashSet::new();
        for (edge, &count) in self.edges.iter() {
            let percentage = percent(count, self.total);
            try!(write!(out, "  n{} -> n{} [label=\"{}%\"];\n",
                        edge.caller.0, edge.callee.0, percentage));
            node_ids.insert(edge.caller);
            node_ids.insert(edge.callee);
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

impl AddFrames for CallGraph {
    fn add_frames<I>(&mut self, frames: I)
        where I: Iterator<Item=String>
    {
        let mut node_ids: Vec<_> = frames.map(|frame| self.node_id(frame)).collect();

        // just ignore empty samples
        if node_ids.len() == 0 {
            return;
        }

        self.frames.reserve(node_ids.len() + 1);
        self.frames.extend(node_ids.iter().cloned());
        self.frames.push(MARKER);

        node_ids.sort();
        node_ids.dedup();

        for id in node_ids {
            self.node_counts[id.0] += 1;
        }
    }
}
