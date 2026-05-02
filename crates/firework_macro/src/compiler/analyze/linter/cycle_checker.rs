// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use petgraph::algo::astar;
use petgraph::prelude::*;

use std::collections::HashMap;

pub struct CycleChecker {
    graph: DiGraph<usize, ()>,
    nodes: HashMap<usize, NodeIndex>,
}

impl CycleChecker {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.graph = DiGraph::new();
        self.nodes = HashMap::new();
    }

    pub fn add_spark(&mut self, id: usize) {
        let node = self.graph.add_node(id);
        self.nodes.insert(id, node);
    }

    pub fn depend(&mut self, id_parent: usize, id_child: usize) -> Option<Vec<usize>> {
        if let (Some(&p), Some(&c)) = (self.nodes.get(&id_parent), self.nodes.get(&id_child)) {
            if let Some((_, path_nodes)) = astar(&self.graph, c, |finish| finish == p, |_| 1, |_| 0)
            {
                let mut cycle_ids = Vec::new();

                for node_idx in path_nodes {
                    for (&id, &n) in self.nodes.iter() {
                        if n == node_idx {
                            cycle_ids.push(id);
                            break;
                        }
                    }
                }

                cycle_ids.insert(0, id_parent);
                return Some(cycle_ids);
            }

            self.graph.add_edge(p, c, ());
        }

        None
    }
}
