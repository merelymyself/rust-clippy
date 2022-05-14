use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::ty::is_type_diagnostic_item;
use clippy_utils::{match_qpath, paths, peel_hir_expr_refs};
use rustc_hir::{StmtKind,BorrowKind, Mutability, BindingAnnotation, PatKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::sym;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```rust
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.62.0"]
    pub PATH_FROM_FORMAT,
    pedantic,
    "default lint description"
}

declare_lint_pass!(PathFromFormat => [PATH_FROM_FORMAT]);

impl<'tcx> LateLintPass<'tcx> for PathFromFormat {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if_chain! {
            if let ExprKind::Call(func, args) = expr.kind;
            if let ExprKind::Path(ref qpath) = func.kind;
            if match_qpath(qpath, &["PathBuf", "from"]);
            // if args.len() == 1;
            if let Some(macro_def_id) = args[0].span.ctxt().outer_expn_data().macro_def_id;
            if cx.tcx.get_diagnostic_name(macro_def_id) == Some(sym::format_macro);
            // if let ExprKind::Block(block, None) = args[0].kind;
            // if block.stmts.len() == 1;
            // if let StmtKind::Local(local) = block.stmts[0].kind;
            // if let Some(init) = local.init;
            // if let ExprKind::Call(func1, args1) = init.kind;
            // if let ExprKind::Path(ref qpath1) = func1.kind;
            // if match_qpath(qpath1, &["$crate", "fmt", "format"]);
            // if args1.len() == 1;
            // if let ExprKind::Call(func2, args2) = args1[0].kind;
            // if let ExprKind::Path(ref qpath2) = func2.kind;
            // if match_qpath(qpath2, &["$crate", "fmt", "Arguments", "new_v1"]);
            // if args2.len() == 2;
            // if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Not, inner) = args2[0].kind;
            // if let ExprKind::Array(elements) = inner.kind;
            // if elements.len() == 2;
            // if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Not, inner1) = args2[1].kind;
            // if let ExprKind::Array(elements1) = inner1.kind;
            // if elements1.len() == 1;
            // if let ExprKind::Call(func3, args3) = elements1[0].kind;
            // if let ExprKind::Path(ref qpath3) = func3.kind;
            // if match_qpath(qpath3, &["$crate", "fmt", "ArgumentV1", "new_display"]);
            // if args3.len() == 1;
            // if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Not, inner2) = args3[0].kind;
            // if let ExprKind::Path(ref qpath4) = inner2.kind;
            // if match_qpath(qpath4, &["base_path"]);
            // if let PatKind::Binding(BindingAnnotation::Unannotated, _, name, None) = local.pat.kind;
            // if name.as_str() == "res";
            // if let Some(trailing_expr) = block.expr;
            // if let ExprKind::Path(ref qpath5) = trailing_expr.kind;
            // if match_qpath(qpath5, &["res"]);
            then {    
                span_lint_and_help(
                    cx,
                    PATH_FROM_FORMAT,
                    expr.span,
                    "`format!(..)` used to form `PathBuf`",
                    None,
                    "consider using `.join()` or `.push()` to avoid the extra allocation",
                );
               // report your lint here
            }
        }
    }
}   
