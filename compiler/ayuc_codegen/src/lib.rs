use ayuc_ast::{
    Ast,
    node::{Node, decl::Declaration, expr::Expression, stmt::Statement},
};

pub struct LuauCodegen;

impl LuauCodegen {
    pub(crate) fn emit_nodes(nodes: &[Node]) -> String {
        let mut result = String::new();

        for node in nodes {
            match node {
                Node::Decl(Declaration::Function(decl)) => {
                    result.push_str(&format!("function {}()", decl.ident.sym.as_str()));

                    result.push_str(&Self::emit_nodes(&decl.block.children));

                    result.push_str("end");
                }
                Node::Stmt(Statement::Expr(Expression::Call(call))) => {
                    result.push_str(&format!("{}();", call.callee.sym.as_str()));
                }
                _ => todo!(),
            }
        }

        result
    }

    pub fn emit(ast: &Ast) -> String {
        let mut result = String::new();

        result.push_str(&Self::emit_nodes(&ast.nodes));

        result
    }
}
