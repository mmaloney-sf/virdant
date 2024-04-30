use virdant::context::Context;
use virdant::parse::{parse_package, parse_expr};
use virdant::value::*;
use virdant::sim::*;
use virdant::hir::*;
use virdant::common::*;
use virdant::types::Type;

fn main() {
    hir_package();
}

pub fn mlir() {
    let package_text = "

        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8] := in;
            reg buf : Word[8] on clk <= in->add(1);
        }

    ";
    let package_ast = parse_package(package_text).unwrap();
    let package = Package::from_ast(&package_ast);
    println!("{package_text}");
    let mut stdout = std::io::stdout();
    package.mlir(&mut stdout).unwrap();
}

pub fn hir_package() {
    let package_text = "

        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];

            submodule buf of Buffer;
            buf.in := in;

            out := buf.out->add(in);
        }

        module Buffer {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8] := in;
            reg buf : Word[8] on clk <= in;
        }

    ";
    let package = parse_package(package_text).unwrap();
    println!("{package_text}");
    let package = Package::compile(&package).unwrap();
    dbg!(&package);

//    let mut stdout = std::io::stdout();
//    package.mlir(&mut stdout).unwrap();
}

pub fn parse() {
    let package = parse_package("

        public module Top {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8];

            submodule buf of Buffer;
            buf.in := in;

            out := buf.out->add(in);
        }

        module Buffer {
            incoming clk : Clock;
            incoming in : Word[8];
            outgoing out : Word[8] := in;
            reg buf : Word[8] on clk <= in;
        }

    ").unwrap();
    dbg!(package);
}

pub fn sim() {
    let ctx = Context::from(vec![
        ("r".into(), Type::Word(8).into()),
        ("in".into(), Type::Word(8).into()),
        ("out".into(), Type::Word(8).into()),
    ]);

    let out_expr: Expr = Expr::from_ast(&parse_expr("r").unwrap()).typeinfer(ctx.clone()).unwrap();
    let in_expr: Expr = Expr::from_ast(&parse_expr("1w8").unwrap()).typeinfer(ctx.clone()).unwrap();
    let r_expr: Expr = Expr::from_ast(&parse_expr("r->add(in)").unwrap()).typeinfer(ctx.clone()).unwrap();

    let mut sim = Sim::new()
        .add_simple_node("top.out".into(), out_expr)
        .add_simple_node("top.in".into(), in_expr)
        .add_reg_node("top.r".into(), Type::Word(8).into(), Some(Value::Word(8, 100)), r_expr)
        .build();

    println!("################################################################################");
    println!("Initial");
    println!("{sim}");

    sim.reset();
    println!("################################################################################");
    println!("reset");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");

    sim.poke("top.in".into(), Value::Word(8, 10));
    println!("poke top.in = 10w8");
    println!("################################################################################");
    println!("{sim}");

    sim.clock();
    println!("################################################################################");
    println!("clock");
    println!("{sim}");
}

pub fn repl() {
    loop {
        let mut input = String::new();
        print!(">>> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match parse_expr(&input) {
                Ok(expr) => {
                    let ctx = Context::from(vec![
                        ("x".into(), Value::Word(8, 1)),
                        ("y".into(), Value::Word(8, 2)),
                        ("z".into(), Value::Word(8, 4)),
                        ("zero".into(), Value::Word(8, 0)),
                        ("buffer.out".into(), Value::Word(8, 42)),
                    ]);
                    let type_ctx = value_context_to_type_context(ctx.clone());
                    let untyped_expr = Expr::from_ast(&expr);
                    match untyped_expr.typeinfer(type_ctx) {
                        Err(e) => eprintln!("ERROR: {e:?}"),
                        Ok(typed_expr) => println!("{}", typed_expr.eval(ctx)),
                    }
                },
                Err(err) => eprintln!("{err:?}"),
            }
        }
    }
}

pub fn value_context_to_type_context(ctx: Context<Path, Value>) -> Context<Path, Arc<Type>> {
    let new_ctx: Vec<(Path, Arc<Type>)> = ctx.into_inner().into_iter().map(|(path, value)| (path, value.type_of().into())).collect();
    Context::from(new_ctx)
}
