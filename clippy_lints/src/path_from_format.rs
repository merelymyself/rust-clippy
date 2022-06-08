use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::macros::{root_macro_call, FormatArgsExpn};
use clippy_utils::sugg::Sugg;
use clippy_utils::ty::is_type_diagnostic_item;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::sym;

declare_clippy_lint! {
    /// ### What it does
    /// Checks for `PathBuf::From(format!(..))` calls.
    ///
    /// ### Why is this bad?
    /// It is not OS-agnostic, and can be harder to read.
    ///
    /// ### Example
    /// ```rust
    /// use std::path::PathBuf;
    /// let base_path = "/base";
    /// PathBuf::from(format!("{}/foo/bar", base_path));
    /// ```
    /// Use instead:
    /// ```rust
    /// use std::path::Path;
    /// let base_path = "/base";
    /// Path::new(base_path).join("foo").join("bar");
    /// ```
    #[clippy::version = "1.62.0"]
    pub PATH_FROM_FORMAT,
    pedantic,
    "builds a `PathBuf` from a format macro"
}

declare_lint_pass!(PathFromFormat => [PATH_FROM_FORMAT]);

impl<'tcx> LateLintPass<'tcx> for PathFromFormat {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if_chain! {
            if let ExprKind::Call(_, args) = expr.kind;
            if let ty = cx.typeck_results().expr_ty(expr);
            if is_type_diagnostic_item(cx, ty, sym::PathBuf);
            if !args.is_empty();
            if let Some(macro_call) = root_macro_call(args[0].span);
            if cx.tcx.item_name(macro_call.def_id) == sym::format;
            if let Some(format_args) = FormatArgsExpn::find_nested(cx, &args[0], macro_call.expn);
            let format_string_parts = format_args.format_string_parts;
            let format_value_args = format_args.value_args;
            then {
                let mut string_parts: Vec<&str> = format_string_parts.iter().map(|x| x.as_str()).collect();
                string_parts.push("");
                let mut applicability = Applicability::MachineApplicable;
                let real_vars: Vec<Sugg<'_>> = format_value_args.iter().map(|x| Sugg::hir_with_applicability(cx, x, "..", &mut applicability)).collect();
                let order_of_real_vars: Vec<usize> = format_args.formatters.iter().map(|(x, _)| x.clone()).collect();
                let mut sugg = String::new();
                for n in 0..real_vars.len() {
                    if n == 0 {
                        if string_parts[0].is_empty() {
                            sugg = format!("Path::new({})", real_vars[order_of_real_vars[0]]);
                        }
                        else {
                            sugg = format!("Path::new(\"{}\").join({y})", string_parts[0], y = real_vars[order_of_real_vars[0]]);
                        }
                    }
                    else {
                        if string_parts[n].is_empty() {
                            sugg = format!("{sugg}.join({})", real_vars[order_of_real_vars[n]]);
                        }
                        else {
                            let mut string = String::from(string_parts[n]);
                            if string.starts_with('/') || string.starts_with('\\') {
                                string.remove(0);
                            }
                            sugg = format!("{sugg}.join(\"{}\").join({y})", string, y = real_vars[order_of_real_vars[n]]);
                        }
                    }
                }
                if !string_parts[real_vars.len()].is_empty() {
                    let mut string = String::from(string_parts[real_vars.len()]);
                    if string.starts_with('/') || string.starts_with('\\') {
                        string.remove(0);
                    }
                    sugg = format!("{sugg}.join(\"{}\")", string);
                }
                span_lint_and_sugg(
                    cx,
                    PATH_FROM_FORMAT,
                    expr.span,
                    "`format!(..)` used to form `PathBuf`",
                    "consider using `.join()` to avoid the extra allocation",
                    sugg,
                    Applicability::MaybeIncorrect,
                );
            }
        }
    }
}
