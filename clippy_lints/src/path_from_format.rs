use clippy_utils::diagnostics::{span_lint_and_sugg, span_lint_and_help};
use clippy_utils::source::snippet_with_applicability;
use clippy_utils::ty::is_type_diagnostic_item;
use clippy_utils::macros::{root_macro_call, FormatArgsExpn};
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
            let mut applicability = Applicability::MachineApplicable;
            let format_args_snip = snippet_with_applicability(cx, format_args.inputs_span(), "..", &mut applicability);
            if let Some(end_of_literal) = format_args_snip.find("\",");
            then {
                let (literal, vars) = format_args_snip.split_at(end_of_literal);
                let mut literal = literal.to_string();
                literal.remove(0);
                let v: Vec<&str> = literal.split("{}").collect();
                let real_vars = vars.strip_prefix("\", ").unwrap_or(vars);
                if v.len() != 2 || real_vars.contains(',') {
                    span_lint_and_help(
                        cx,
                        PATH_FROM_FORMAT,
                        expr.span,
                        "`format!(..)` used to form `PathBuf`",
                        None,
                        "consider using `.join()` to avoid the extra allocation",
                    ); 
                    return;
                }
                let sugg = {
                    if v[0].is_empty() {
                        let mut str1 = v[1].to_string();
                        if str1.starts_with('\\') || str1.starts_with('/') {
                            str1.remove(0);
                        }
                        format!("Path::new({real_vars}).join(\"{str1}\")")
                    }
                    else if v[1].is_empty() {
                        let str1 = v[0].to_string();
                        format!("Path::new(\"{str1}\").join({real_vars})")
                    }
                    else {
                        let (str1, mut str2) = (v[0].to_string(), v[1].to_string());
                        if str2.starts_with('\\') || str2.starts_with('/') {
                            str2.remove(0);
                        }
                        format!("Path::new(\"{str1}\").join({real_vars}).join(\"{str2}\")")
                    }
                };
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
