use crate::{ast::fn_call::FnCall, error::CompileError, Compile, Compiler, CompilerResult};
use inkwell::values::FloatValue;
use rogato_common::ast::expression::Expression;

impl<'ctx> Compile<'ctx, FloatValue<'ctx>> for Expression {
    fn compile(&self, c: &'ctx mut Compiler) -> CompilerResult<FloatValue<'ctx>> {
        match self {
            Expression::Commented(_c, e) => e.compile(c),
            Expression::Lit(lit_exp) => lit_exp.compile(c),
            Expression::FnCall(fn_ident, args) => {
                let fn_call = FnCall::new(fn_ident, args);
                fn_call.compile(c)
            }
            Expression::OpCall(op_ident, left, right) => {
                let comp = c;
                let left = left.compile(comp)?;
                let right = right.compile(comp)?;
                match op_ident.as_str() {
                    "+" => Ok(c.builder().build_float_add(left, right, "tmpadd")),
                    "-" => Ok(c.builder().build_float_sub(left, right, "tmpsub")),
                    "*" => Ok(c.builder().build_float_mul(left, right, "tmpmul")),
                    "/" => Ok(c.builder().build_float_div(left, right, "tmpdiv")),
                    _ => Err(CompileError::OpNotDefined(op_ident.clone())),
                }
            }
            Expression::Var(id) => match c.lookup_var(id.as_str()) {
                Some(var) => Ok(c.builder().build_load(*var, id.as_str()).into_float_value()),
                None => Err(CompileError::VarNotFound(id.clone())),
            },
            Expression::ConstOrTypeRef(_id) => todo!(),
            Expression::DBTypeRef(_id) => todo!(),
            Expression::PropFnRef(_id) => todo!(),
            Expression::EdgeProp(_id, _edge) => todo!(),
            Expression::IfElse(if_else) => if_else.compile(c),
            Expression::Let(let_expr) => let_expr.compile(c),
            Expression::Lambda(lambda) => lambda.compile(c),
            Expression::Query(query) => query.compile(c),
            Expression::Symbol(_id) => todo!(),
            Expression::Quoted(_expr) => todo!(),
            Expression::QuotedAST(_ast) => todo!(),
            Expression::Unquoted(_expr) => todo!(),
            Expression::UnquotedAST(_ast) => todo!(),
            Expression::InlineFnDef(fn_def) => fn_def.compile(c),
        }
    }
}
