use virdant::context::Context;
use virdant::parse::{parse_package, parse_expr};
use virdant::value::*;
use virdant::hir::*;
use virdant::common::*;
use virdant::types::Type;
use virdant::db;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "virdant", author, version, about, long_about = None)]
struct Args {
    filename: String,

    #[arg(short, long, default_value_t = false)]
    compile: bool,

    #[arg(long)]
    sim: bool,

    #[arg(long)]
    top: Option<String>,

    #[arg(long)]
    delay: Option<usize>,

    #[arg(long)]
    mlir: bool,

    #[arg(long)]
    trace: Option<String>,
}

fn main() {
    let args = Args::parse();
    if args.compile {
        let package_text = std::fs::read_to_string(args.filename).unwrap();
        todo!()
//        db::compile_verilog(&package_text).unwrap();
    } else if args.sim {
        let top = args.top.unwrap_or_else(|| "Top".into());
        let trace = args.trace.as_ref().map(|s| s.as_str());
        sim(&args.filename, &top, trace, args.delay.unwrap_or(400));
    } else if args.mlir {
        let package_text = std::fs::read_to_string(args.filename).unwrap();
        todo!()
//        db::compile_mlir(&package_text).unwrap();
    } else {
        eprintln!("Please specify either --sim or --compile.");
    }
}

pub fn sim(filename: &str, top: &str, trace: Option<&str>, delay: usize) {
    let package = std::fs::read_to_string(filename).unwrap();

    let mut sim = if let Some(trace) = trace {
    let mut fout = std::fs::File::create(trace).unwrap();
        virdant::sim::simulator_with_trace(&package, top, &mut fout).unwrap()
    } else {
        virdant::sim::simulator(&package, top).unwrap()
    };

    sim.poke("top.reset".into(), Value::Word(1, 1));
    println!("################################################################################");
    println!("Initial");
    println!("{sim}");

    sim.clock();
    sim.poke("top.reset".into(), Value::Word(1, 0));
    println!("################################################################################");
    println!("reset");
    println!("{sim}");

    loop {
        sim.clock();
        println!("################################################################################");
        println!("clock");
        println!("{sim}");

        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay as u64));
        }
    }
}

pub fn verilog() {
    let package_text = "

        public module Top {
            incoming clk : Clock;
            incoming in  : Word[8];
            outgoing out : Word[8];

            reg b : Word[8] on clk
                <= in->add(1);

            submodule buffer of Buffer;
            buffer.clk := clk;
            buffer.in := b;

            out := buffer.out;
        }

        module Buffer {
            incoming clk : Clock;
            incoming in  : Word[8];
            outgoing out : Word[8];

            reg b : Word[8] on clk <= in;
            out := b;
        }

    ";
    todo!()
//    db::compile_verilog(package_text).unwrap();
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
