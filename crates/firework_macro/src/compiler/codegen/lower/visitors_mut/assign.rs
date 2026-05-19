// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::*;

use super::LowerVisitor;

impl LowerVisitor<'_> {
    /// Удаляет обёртку derived!() из выражения присваивания
    pub(crate) fn lower_assign_derived(&mut self, i: &mut ExprAssign) {
        if let Expr::Macro(macro_expr) = &mut *i.right
            && macro_expr.mac.path.is_ident("derived")
            && let Ok(inner_expr) = syn::parse2::<Expr>(macro_expr.mac.tokens.clone())
        {
            *i.right = inner_expr;
        }
    }

    /// Удаляет обёртку derived!() из бинарной операции с присваиванием
    pub(crate) fn lower_binary_derived(&mut self, i: &mut ExprBinary) {
        let is_assign_op = matches!(
            i.op,
            BinOp::AddAssign(_)
                | BinOp::SubAssign(_)
                | BinOp::MulAssign(_)
                | BinOp::DivAssign(_)
                | BinOp::RemAssign(_)
                | BinOp::BitAndAssign(_)
                | BinOp::BitOrAssign(_)
                | BinOp::BitXorAssign(_)
                | BinOp::ShlAssign(_)
                | BinOp::ShrAssign(_)
        );

        if is_assign_op
            && let Expr::Macro(macro_expr) = &mut *i.right
            && macro_expr.mac.path.is_ident("derived")
            && let Ok(inner_expr) = syn::parse2::<Expr>(macro_expr.mac.tokens.clone())
        {
            *i.right = inner_expr;
        }
    }
}
