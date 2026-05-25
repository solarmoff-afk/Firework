// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro_warning::FormattedWarning;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use super::FireworkLinter;

impl FireworkLinter {
    pub fn generate_cycle_warning(
        &self,
        id_parent: usize,
        id_child: usize,
        cycle_path: &[usize],
        span: Span,
    ) -> TokenStream {
        let parent_name = self
            .nodes_map
            .get(&id_parent)
            .map(|s| s.0.as_str())
            .unwrap_or("unknown");

        let child_name = self
            .nodes_map
            .get(&id_child)
            .map(|s| s.0.as_str())
            .unwrap_or("unknown");

        let child_code = self
            .nodes_map
            .get(&id_child)
            .map(|s| s.1.as_str())
            .unwrap_or("unknown");

        let flat_path = cycle_path
            .iter()
            .map(|&id| {
                let name = self
                    .nodes_map
                    .get(&id)
                    .map(|s| s.0.as_str())
                    .unwrap_or("unknown");

                format!("{} ({})", name, id)
            })
            .collect::<Vec<_>>()
            .join(" -> ");

        let mut ascii_graph = String::new();
        let unique_nodes = &cycle_path[0..cycle_path.len() - 1];

        ascii_graph.push_str(&format!(
            "             ╭──▶ {} (id {})\n",
            parent_name, id_parent
        ));

        for &node_id in &unique_nodes[1..] {
            let node_name = self
                .nodes_map
                .get(&node_id)
                .map(|s| s.0.as_str())
                .unwrap_or("unknown");

            ascii_graph.push_str("             │      ▼\n");
            ascii_graph.push_str(&format!(
                "             │    {} (id {})\n",
                node_name, node_id
            ));
        }

        ascii_graph.push_str("             ╰──────┘");

        let nodes_count = unique_nodes.len();
        let message = format!(
            "\
FW001: cyclic dependency detected between reactive variables
   = note: variable `{parent_name}` (id {id_parent}) depends on `{child_name}` (id {id_child}),
           which ultimately depends back on `{parent_name}`, creating an infinite reactive loop.
   = help: the code generator will prevent a runtime freeze by capping iterations at 64,
           however, this usually indicates a flaw in your reactive logic and should be fixed.
   = note: With the variable {parent_name}, the variable initialized at this location is closed
           in a cyclic dependency: {child_code}
   = note: distance between `{parent_name}` and `{parent_name}` is {nodes_count} nodes:
           {flat_path}
   = note: dependency graph:
{ascii_graph}"
        );

        let warning = FormattedWarning::new_deprecated(
            format!("FW001_{}", self.counter).as_str(),
            message,
            span,
        );

        warning.into_token_stream()
    }
}
