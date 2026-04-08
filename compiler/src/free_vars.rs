use std::collections::HashSet;
use std::ops::Deref;

use rogato_common::ast::{
    expression::Expression,
    fn_def::FnDefBody,
    lambda::{Lambda, LambdaVariant},
    pattern::Pattern,
    VarIdentifier,
};

/// Collects all free variables in a lambda — variables referenced in the body
/// that are not defined as lambda arguments or locally bound (via let expressions).
pub fn collect_free_vars(lambda: &Lambda) -> HashSet<VarIdentifier> {
    let mut free_vars = HashSet::new();

    for variant in lambda.variants_iter() {
        let mut bound = HashSet::new();
        // Collect argument-bound variables
        for arg_pattern in variant.args.iter() {
            collect_pattern_vars(arg_pattern, &mut bound);
        }
        // Analyze the body for free variables
        collect_expr_free_vars(&variant.body, &bound, &mut free_vars);
    }

    free_vars
}

/// Collects free variables in a single lambda variant (useful when compiling individual variants).
pub fn collect_variant_free_vars(variant: &LambdaVariant) -> HashSet<VarIdentifier> {
    let mut free_vars = HashSet::new();
    let mut bound = HashSet::new();

    for arg_pattern in variant.args.iter() {
        collect_pattern_vars(arg_pattern, &mut bound);
    }

    collect_expr_free_vars(&variant.body, &bound, &mut free_vars);
    free_vars
}

/// Extracts all variable names bound by a pattern.
fn collect_pattern_vars(pattern: &Pattern, bound: &mut HashSet<VarIdentifier>) {
    match pattern {
        Pattern::Var(id) => {
            bound.insert(id.clone());
        }
        Pattern::ListCons(head, tail) => {
            collect_pattern_vars(head, bound);
            collect_pattern_vars(tail, bound);
        }
        Pattern::List(items) | Pattern::Tuple(_, items) => {
            for item in items.iter() {
                collect_pattern_vars(item, bound);
            }
        }
        Pattern::Map(kv_pairs) => {
            for kv in kv_pairs.iter() {
                collect_pattern_vars(&kv.value, bound);
            }
        }
        Pattern::MapCons(kv_pairs, rest) => {
            for kv in kv_pairs.iter() {
                collect_pattern_vars(&kv.value, bound);
            }
            collect_pattern_vars(rest, bound);
        }
        // These patterns don't bind any variables
        Pattern::Any
        | Pattern::EmptyList
        | Pattern::Bool(_)
        | Pattern::Number(_)
        | Pattern::String(_)
        | Pattern::Symbol(_) => {}
    }
}

/// Recursively collects free variables from an expression.
/// `bound` contains all variables that are in scope (lambda args, let bindings, etc.)
/// `free` accumulates free variables found.
fn collect_expr_free_vars(
    expr: &Expression,
    bound: &HashSet<VarIdentifier>,
    free: &mut HashSet<VarIdentifier>,
) {
    match expr {
        Expression::Var(id) => {
            if !bound.contains(id) {
                free.insert(id.clone());
            }
        }
        Expression::Lit(_) => {
            // Literals don't reference variables
        }
        Expression::FnCall(fn_call) => {
            // Don't treat the function identifier as a free variable — it's resolved separately.
            // Only analyze the arguments.
            for arg in fn_call.args.iter() {
                collect_expr_free_vars(arg, bound, free);
            }
        }
        Expression::OpCall(_op, left, right) => {
            collect_expr_free_vars(left, bound, free);
            collect_expr_free_vars(right, bound, free);
        }
        Expression::IfElse(if_else) => {
            collect_expr_free_vars(&if_else.condition, bound, free);
            collect_expr_free_vars(&if_else.then_expr, bound, free);
            collect_expr_free_vars(&if_else.else_expr, bound, free);
        }
        Expression::Let(let_expr) => {
            let mut inner_bound = bound.clone();
            for (var_id, val_expr) in let_expr.bindings.iter() {
                // The binding's value expression is evaluated in the outer scope
                collect_expr_free_vars(val_expr, bound, free);
                inner_bound.insert(var_id.clone());
            }
            // The body is evaluated with the new bindings in scope
            collect_expr_free_vars(&let_expr.body, &inner_bound, free);
        }
        Expression::Lambda(inner_lambda) => {
            let inner_lambda = inner_lambda.deref();
            // For nested lambdas, their free variables that aren't in our bound set
            // become our free variables too.
            for variant in inner_lambda.variants_iter() {
                let mut inner_bound = bound.clone();
                for arg_pattern in variant.args.iter() {
                    collect_pattern_vars(arg_pattern, &mut inner_bound);
                }
                collect_expr_free_vars(&variant.body, &inner_bound, free);
            }
        }
        Expression::InlineFnDef(fn_def) => {
            let fn_def = fn_def.borrow();
            // The function name itself becomes bound (for recursive references)
            let mut inner_bound = bound.clone();
            inner_bound.insert(fn_def.id().clone().into());
            for variant in fn_def.variants_iter() {
                let mut variant_bound = inner_bound.clone();
                for arg_pattern in variant.0.iter() {
                    collect_pattern_vars(arg_pattern, &mut variant_bound);
                }
                match variant.1.deref() {
                    FnDefBody::RogatoFn(body) => {
                        collect_expr_free_vars(body, &variant_bound, free);
                    }
                    FnDefBody::NativeFn(_) => {}
                }
            }
        }
        Expression::Commented(_, inner_expr) => {
            collect_expr_free_vars(inner_expr, bound, free);
        }
        Expression::Quoted(inner_expr) => {
            collect_expr_free_vars(inner_expr, bound, free);
        }
        Expression::Unquoted(inner_expr) => {
            collect_expr_free_vars(inner_expr, bound, free);
        }
        Expression::EdgeProp(inner_expr, _edge) => {
            collect_expr_free_vars(inner_expr, bound, free);
        }
        // These don't reference variables in a way that constitutes free variable capture
        Expression::ConstOrTypeRef(_)
        | Expression::DBTypeRef(_)
        | Expression::PropFnRef(_)
        | Expression::Symbol(_)
        | Expression::QuotedAST(_)
        | Expression::UnquotedAST(_) => {}
        Expression::Query(_) => {
            // TODO: Query expressions have their own binding structure (QueryBindings)
            // and would require deeper analysis. For now, treat as opaque.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rogato_common::ast::{
        expression::Expression,
        fn_call::{FnCall, FnCallArgs},
        if_else::IfElse,
        lambda::{Lambda, LambdaArgs, LambdaVariant},
        let_expression::{LetBindings, LetExpression},
        literal::Literal,
    };
    use std::rc::Rc;

    fn var_expr(name: &str) -> Rc<Expression> {
        Rc::new(Expression::Var(VarIdentifier::from(name)))
    }

    fn num_expr(n: i32) -> Rc<Expression> {
        Rc::new(Expression::Lit(Literal::Number(n.into())))
    }

    fn var_pattern(name: &str) -> Rc<Pattern> {
        Rc::new(Pattern::Var(VarIdentifier::from(name)))
    }

    fn make_lambda(args: Vec<&str>, body: Rc<Expression>) -> Lambda {
        let arg_patterns: Vec<Rc<Pattern>> = args.into_iter().map(var_pattern).collect();
        let variant = Rc::new(LambdaVariant::new(LambdaArgs::new(arg_patterns), body));
        Lambda::new(vec![variant])
    }

    #[test]
    fn no_free_vars_simple() {
        // (x -> x)
        let lambda = make_lambda(vec!["x"], var_expr("x"));
        let free = collect_free_vars(&lambda);
        assert!(free.is_empty());
    }

    #[test]
    fn one_free_var() {
        // (x -> x + y) — y is free
        let body = Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y")));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("y")));
    }

    #[test]
    fn multiple_free_vars() {
        // (x -> (x + y) + z) — y and z are free
        let inner = Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y")));
        let body = Rc::new(Expression::OpCall("+".into(), inner, var_expr("z")));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 2);
        assert!(free.contains(&VarIdentifier::from("y")));
        assert!(free.contains(&VarIdentifier::from("z")));
    }

    #[test]
    fn let_binding_shadows() {
        // (x -> let y = 5 in x + y) — no free vars, y is locally bound
        let let_expr = LetExpression::new(
            LetBindings::new(vec![(VarIdentifier::from("y"), num_expr(5))]),
            Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y"))),
        );
        let body = Rc::new(Expression::Let(let_expr));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert!(free.is_empty());
    }

    #[test]
    fn let_binding_value_uses_outer_scope() {
        // (x -> let y = z in x + y) — z is free (referenced in binding value)
        let let_expr = LetExpression::new(
            LetBindings::new(vec![(VarIdentifier::from("y"), var_expr("z"))]),
            Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y"))),
        );
        let body = Rc::new(Expression::Let(let_expr));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("z")));
    }

    #[test]
    fn nested_lambda_captures() {
        // (x -> (y -> x + y + z)) — z is free, x comes from outer lambda
        let inner_body = Rc::new(Expression::OpCall(
            "+".into(),
            Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y"))),
            var_expr("z"),
        ));
        let inner_lambda = make_lambda(vec!["y"], inner_body);
        let body = Rc::new(Expression::Lambda(Rc::new(inner_lambda)));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        // z is free (not bound by either lambda)
        // x is NOT free (bound by outer lambda)
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("z")));
    }

    #[test]
    fn fn_call_args_analyzed() {
        // (x -> someFunc y x) — y is free
        let fn_call = FnCall::new(
            "someFunc".into(),
            FnCallArgs::new(vec![var_expr("y"), var_expr("x")]),
        );
        let body = Rc::new(Expression::FnCall(fn_call));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("y")));
    }

    #[test]
    fn fn_call_name_not_captured() {
        // (x -> someFunc x) — someFunc is NOT treated as a free variable
        let fn_call = FnCall::new("someFunc".into(), FnCallArgs::new(vec![var_expr("x")]));
        let body = Rc::new(Expression::FnCall(fn_call));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert!(free.is_empty());
    }

    #[test]
    fn if_else_analyzed() {
        // (x -> if cond then x else y) — cond and y are free
        let if_else = IfElse::new(var_expr("cond"), var_expr("x"), var_expr("y"));
        let body = Rc::new(Expression::IfElse(if_else));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 2);
        assert!(free.contains(&VarIdentifier::from("cond")));
        assert!(free.contains(&VarIdentifier::from("y")));
    }

    #[test]
    fn multiple_args_bound() {
        // (x y -> x + y) — no free vars
        let body = Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y")));
        let lambda = make_lambda(vec!["x", "y"], body);
        let free = collect_free_vars(&lambda);
        assert!(free.is_empty());
    }

    #[test]
    fn variant_free_vars() {
        // Test collect_variant_free_vars directly
        let arg_patterns = vec![var_pattern("x")];
        let body = Rc::new(Expression::OpCall("+".into(), var_expr("x"), var_expr("y")));
        let variant = LambdaVariant::new(LambdaArgs::new(arg_patterns), body);
        let free = collect_variant_free_vars(&variant);
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("y")));
    }

    #[test]
    fn literal_no_free_vars() {
        // (x -> 42) — no free vars
        let lambda = make_lambda(vec!["x"], num_expr(42));
        let free = collect_free_vars(&lambda);
        assert!(free.is_empty());
    }

    #[test]
    fn commented_expr_analyzed() {
        // (x -> // comment \n y) — y is free
        let body = Rc::new(Expression::Commented(
            " a comment".to_string(),
            var_expr("y"),
        ));
        let lambda = make_lambda(vec!["x"], body);
        let free = collect_free_vars(&lambda);
        assert_eq!(free.len(), 1);
        assert!(free.contains(&VarIdentifier::from("y")));
    }
}
