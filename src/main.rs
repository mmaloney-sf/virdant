use std::collections::HashSet;

use virdant::common::*;
use virdant::phase;
use virdant::phase::imports::ImportsQ;
use virdant::phase::Db;
use virdant::phase::check::CheckQ;

use clap::Parser;
use virdant::phase::PackageId;

#[derive(Parser, Debug)]
#[command(name = "vir", author, version, about, long_about = None)]
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
    if args.compile || true {

        let path = std::path::PathBuf::from(args.filename.clone());
        let db = load_from_top_source(&path).unwrap();

        db.check().unwrap();

        let mut stdout = std::io::stdout();
        if let Err(e) = db.verilog(&mut stdout) {
            eprintln!("{e:?}");
            std::process::exit(-1);
        }
    } else if args.sim {
        todo!()
    } else if args.mlir {
        todo!()
    } else {
        eprintln!("Please specify either --sim or --compile.");
    }
}

fn load_from_top_source(path: &std::path::Path) -> VirdantResult<Db> {
    let mut db = phase::Db::new();
    let mut queue = vec![];
    let mut imported = HashSet::new();

    let source_dir = path.parent().unwrap();

    let package = import_from_fillepath(&mut db, path);
    imported.insert(package.clone());

    for package in db.package_imports(package)? {
        if !imported.contains(&package) {
            queue.push(package.clone());
        }
    }

    while let Some(package) = queue.pop() {
        let path = source_dir.join(format!("{package}.vir"));
        let package = import_from_fillepath(&mut db, &path);
        for package in db.package_imports(package)? {
            if !imported.contains(&package) {
                queue.push(package.clone());
            }
        }
    }

    Ok(db)
}

fn import_from_fillepath(db: &mut Db, path: &std::path::Path) -> PackageId {
    let package_name = path.file_stem().unwrap().to_string_lossy();
    eprintln!("LOADING PACKAGE: {package_name} ({})", path.to_string_lossy());
    let package_text = std::fs::read_to_string(&path).unwrap();
    db.set_source(&package_name, &package_text)
}
