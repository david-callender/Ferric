use std::fs;
use std::io::stdout;

use ferric::{
    FerricError,
    interpreter::Interpreter,
    lexer::Lexer,
    loc::ProgramSrc,
    parser::{Expr, Parser},
};

fn ferric_main(source: String) -> Result<(), FerricError> {
    let src = ProgramSrc::new(source);

    let stream = src.clone();
    let lexer = Lexer::new(stream.stream(), src.clone());

    let mut parser = Parser::new(lexer);
    let expr = parser.parse()?;

    for e in &expr {
        walk(src.clone(), e);
    }

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output);
    interpreter.interpret(&expr)?;

    Ok(())
}

fn walk(src: ProgramSrc, e: &Expr) {
    println!("{}\n", e.span.format(&src, "this expression"));

    match &e.kind {
        ferric::parser::ExprKind::Literal(runtime_val) => {}
        ferric::parser::ExprKind::Ident(_) => {}
        ferric::parser::ExprKind::Binary {
            left,
            operation,
            right,
        } => {
            walk(src.clone(), left);
            walk(src.clone(), right);
        }
        ferric::parser::ExprKind::Unary { operation, right } => {
            walk(src.clone(), right);
        }
        ferric::parser::ExprKind::Call { callee, args } => {
            walk(src.clone(), callee);
            for arg in args {
                walk(src.clone(), arg);
            }
        }
        ferric::parser::ExprKind::Decl { value } => {
            walk(src.clone(), value);
        }
        ferric::parser::ExprKind::VarGet { depth, slot } => {}
        ferric::parser::ExprKind::VarSet { value, depth, slot } => {
            walk(src.clone(), value);
        }
        ferric::parser::ExprKind::Block(exprs) => {
            println!("start block");
            for expr in exprs {
                walk(src.clone(), expr);
            }
            println!("end block");
        }
        ferric::parser::ExprKind::If {
            cond,
            then,
            otherwise,
        } => {
            walk(src.clone(), cond);
            for t in then {
                walk(src.clone(), t);
            }
            if let Some(o) = otherwise {
                walk(src.clone(), o);
            }
        }
        ferric::parser::ExprKind::While { cond, body } => {
            walk(src.clone(), cond);
            for b in body {
                walk(src.clone(), b);
            }
        }
        ferric::parser::ExprKind::Func { param_count, body } => {
            for b in body {
                walk(src.clone(), b);
            }
        }
    }
}

fn main() {
    let contents = fs::read_to_string("./executable/src.fe").expect("No such file located.");
    match ferric_main(contents) {
        Ok(()) => {}
        Err(err) => eprintln!("Ferric ran into an error:{err}"),
    }
}
