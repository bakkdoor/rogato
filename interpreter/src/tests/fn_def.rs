use rogato_common::ast::{
    fn_def::{FnDef, FnDefArgs, FnDefBody},
    helpers::{any_p, empty_list_p, fn_call, list_cons_p, number_lit, op_call, var, var_p},
};

#[test]
fn tail_recursive_fn_defs() {
    let rec_fn_def = FnDef::new(
        "count",
        FnDefArgs::new(vec![var_p("acc"), empty_list_p()]),
        FnDefBody::RogatoFn(var("acc")).into(),
    );

    assert!(!rec_fn_def.borrow().is_tail_recursive());

    rec_fn_def.borrow_mut().add_variant(
        FnDefArgs::new(vec![var_p("acc"), list_cons_p(any_p(), var_p("rest"))]),
        FnDefBody::RogatoFn(fn_call(
            "count",
            [op_call("+", var("acc"), number_lit(1)), var("rest")],
        ))
        .into(),
    );

    assert!(rec_fn_def.borrow().is_tail_recursive());
}
