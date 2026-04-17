// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use petgraph::prelude::*;
use petgraph::algo::astar;

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
            
            // Ищем путь от ребенка к родителю ДО добавления ребра
            // Если он есть, значит добавление P -> C замкнет цикл
            if let Some((_, path_nodes)) = astar(&self.graph, c, |finish| finish == p, |_| 1, |_| 0) {
                
                let mut cycle_ids = Vec::new();
                
                // Переводим внутренние индексы графа (NodeIndex) обратно в ваши спарк ID (usize)
                for node_idx in path_nodes {
                    for (&id, &n) in self.nodes.iter() {
                        if n == node_idx {
                            cycle_ids.push(id);
                            break;
                        }
                    }
                }
                
                // Вставляем родителя в начало, чтобы цикл выглядел как: P -> C -> ... -> P
                cycle_ids.insert(0, id_parent);
                return Some(cycle_ids);
            }

            // Если цикла нет, добавляем ребро P -> C
            self.graph.add_edge(p, c, ());
        }
        
        None
    }
}
