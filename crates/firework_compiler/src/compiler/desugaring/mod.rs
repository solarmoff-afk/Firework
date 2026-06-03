// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::visit_mut::VisitMut;
use syn::{Expr, ExprBlock, ExprClosure, File, parse_quote_spanned, spanned::Spanned};

pub struct DesugarVisitor;

impl VisitMut for DesugarVisitor {
    /// Разворачивает короткое замыкание (без блока) в замыкание с блоком внутри. Это
    /// нужно для того, чтобы правильно разворачивать стейтементы let mut cl = || a += 1;
    /// где a это спарк. Сворачивание в блок позволяет кодогенератору правильно сгенерировать
    /// обвязку (в данном случае, изменение битовой маски) для этого выражения, так как
    /// оно теперь в блоке (let mut cl = || a += 1;)
    fn visit_expr_closure_mut(&mut self, i: &mut ExprClosure) {
        syn::visit_mut::visit_expr_closure_mut(self, i);

        if !matches!(*i.body, Expr::Block(_)) {
            let body_expr = &*i.body;
            let span = body_expr.span();

            let new_block: ExprBlock = parse_quote_spanned!(span=>
                { #body_expr }
            );

            *i.body = Expr::Block(new_block);
        }
    }
}

pub fn normalize_ast(file: &mut File) {
    let mut visitor = DesugarVisitor;
    visitor.visit_file_mut(file);
}
