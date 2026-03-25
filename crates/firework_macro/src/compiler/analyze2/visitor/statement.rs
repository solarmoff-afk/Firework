// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    pub(crate) fn analyze_stmt(&mut self, i: &'ast Stmt) {
        let mut layout_name = "".to_string();
        let should_push = if let Stmt::Macro(stmt_macro) = i {
            layout_name = stmt_macro.mac.path.to_token_stream().to_string();
            !is_layout(&layout_name) && !is_widget(&layout_name)
        } else {
            true
        };

        if should_push {
            self.statement.string = i.to_token_stream().to_string();
        } else {
            self.statement.string = format!("{} {{", layout_name);
        }

        // println!("STATEMENT: {}", self.statement_index);
        if let Some(root_id) = self.reactive_block {
            // println!("Statement {} is reactive, start: {}", self.statement_index, root_id.0);
            self.statement.is_reactive_block = true;
        }
        
        visit::visit_stmt(self, i); 
        
        self.statement_index += 1; 
        
        if should_push {
            // Если это лайаут блок то клонирование области видимости и пуш уже
            // были и клонировать второй раз нет смысла
            self.statement.scope = self.scope.clone();
            self.ir.statements.push(self.statement.clone());
        }
        
        self.statement.index = self.statement_index;
        self.statement.action = FireworkAction::DefaultCode;
        self.statement.is_reactive_block = false;
    }
}
