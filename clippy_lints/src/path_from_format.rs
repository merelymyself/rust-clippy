use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::source::snippet;
use clippy_utils::ty::is_type_diagnostic_item;
use clippy_utils::macros::{root_macro_call, FormatArgsExpn};
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::sym;
use std::fmt::Write as _;

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
            then {
                let full_expr = snippet(cx, expr.span, "error").to_string();
                let split_expr: Vec<&str> = full_expr.split('!').collect();
                let args_to_macro = split_expr[1];
                let replaced = args_to_macro.replace('(', "").replace(')', "");
                let unformatted: Vec<&str> = replaced.split(',').collect();
                let mut push_targets: Vec<String> = Vec::new();
                let mut temp_string = String::new();
                for c in unformatted[0].chars() {
                    if c == '/' || c == '\\' {
                        push_targets.push(temp_string.clone());
                        temp_string = String::new();
                    }
                    else if c == '}' {
                        temp_string.push_str(&unformatted[1].replace(' ', ""));
                    }
                    else if c != '{' && c != '"' {
                        temp_string.push(c);
                    }
                }
                if !temp_string.is_empty() {
                    push_targets.push(temp_string.clone());
                    temp_string = String::new();
                }
                for target in push_targets {
                    let target_processed =
                        if target == unformatted[1].replace(' ', "") {
                            target
                        }
                        else {
                            let mut s = String::from("\"");
                            s.push_str(&target);
                            s.push('"');
                            s
                        };
                    if temp_string.is_empty() {
                        let _ = write!(temp_string, "Path::new({})", target_processed);
                    }
                    else {
                        let _ = write!(temp_string, ".join({})", target_processed);
                    }
                }
                span_lint_and_sugg(
                    cx,
                    PATH_FROM_FORMAT,
                    expr.span,
                    "`format!(..)` used to form `PathBuf`",
                    "consider using `.join()` to avoid the extra allocation",
                    temp_string,
                    Applicability::MaybeIncorrect,
                );
            }
        }
    }
}
