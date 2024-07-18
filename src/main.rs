use virdant::common::*;
use virdant::phase;
use virdant::phase::astq::AstQ;
use virdant::phase::check::CheckQ;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "virdant", author, version, about, long_about = None)]
struct Args {
    filenames: Vec<String>,

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
        eprintln!("filenames: {:?}", args.filenames);
        if args.filenames.len() == 0 {
            eprintln!("Requires a filename");
            std::process::exit(-1);
        }

        let mut db = phase::Db::new();

        for filename in &args.filenames {
            let path = std::path::Path::new(&filename);

            let package_name = path.file_stem().unwrap().to_string_lossy();
            eprintln!("LOADING PACKAGE: {package_name}");
            let package_text = std::fs::read_to_string(&path).unwrap();
            db.set_source(&package_name, &package_text);
        }

        db.check().unwrap();

        let mut stdout = std::io::stdout();
        if let Err(e) = db.verilog(&mut stdout) {
            eprintln!("{e:?}");
            std::process::exit(-1);
        }
    } else if args.sim {
//        let top = args.top.unwrap_or_else(|| "Top".into());
//        let trace = args.trace.as_ref().map(|s| s.as_str());
//        sim(&args.filename, &top, trace, args.delay.unwrap_or(400));
        todo!()
    } else if args.mlir {
        /*
        let package_text = std::fs::read_to_string(args.filename).unwrap();
        db::compile_mlir(&package_text).unwrap();
        */
        todo!()
    } else {
        eprintln!("Please specify either --sim or --compile.");
    }
}

/*
pub fn sim(filename: &str, top: &str, trace: Option<&str>, delay: usize) {
    todo!()
    /*
    let package = std::fs::read_to_string(filename).unwrap();

    let mut sim = if let Some(_trace) = trace {
//    let mut fout = std::fs::File::create(trace).unwrap();
//    virdant::sim::simulator_with_trace(&package, top, &mut fout).unwrap()
        todo!()
    } else {
//        virdant::sim::simulator(&package, top).unwrap()
        todo!()
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
*/
}
*/

/*
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
    //db::compile_verilog(package_text).unwrap();
    todo!()
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

pub fn value_context_to_type_context(ctx: Context<Path, Value>) -> Context<Path, Arc<Type>> {
    let new_ctx: Vec<(Path, Arc<Type>)> = ctx.into_inner().into_iter().map(|(path, value)| (path, value.type_of().into())).collect();
    Context::from(new_ctx)
}
*/
