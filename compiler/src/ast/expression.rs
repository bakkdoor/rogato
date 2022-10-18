use crate::{ast::fn_call::FnCall, Compile, Compiler, CompilerResult};
use rogato_common::ast::expression::Expression;

impl Compile<()> for Expression {
    fn compile(&self, context: &mut Compiler) -> CompilerResult<()> {
        match self {
            Expression::Commented(_c, e) => e.compile(context),
            Expression::Lit(lit_exp) => lit_exp.compile(context),
            Expression::FnCall(fn_ident, args) => {
                let fn_call = FnCall::new(fn_ident, args);
                fn_call.compile(context)
            }
            Expression::OpCall(_op_ident, _left, _right) => {
                todo!()
            }
            Expression::Var(_id) => todo!(),
            Expression::ConstOrTypeRef(_id) => todo!(),
            Expression::DBTypeRef(_id) => todo!(),
            Expression::PropFnRef(_id) => todo!(),
            Expression::EdgeProp(_id, _edge) => todo!(),
            Expression::IfElse(if_else) => if_else.compile(context),
            Expression::Let(let_expr) => let_expr.compile(context),
            Expression::Lambda(lambda) => lambda.compile(context),
            Expression::Query(query) => query.compile(context),
            Expression::Symbol(_id) => todo!(),
            Expression::Quoted(_expr) => todo!(),
            Expression::QuotedAST(_ast) => todo!(),
            Expression::Unquoted(_expr) => todo!(),
            Expression::UnquotedAST(_ast) => todo!(),
            Expression::InlineFnDef(fn_def) => fn_def.compile(context),
        }
    }
}
