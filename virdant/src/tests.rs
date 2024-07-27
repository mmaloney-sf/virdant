use crate::parse::*;

const CHECK: char = '✅';
const BATSU: char = '❌';

#[test]
fn parse_examples() {
    let mut errors = vec![];

    for filepath in example_files() {
        let text = std::fs::read_to_string(filepath.clone()).unwrap();

        let filename = filepath
            .file_name()
            .unwrap()
            .to_owned()
            .to_string_lossy()
            .into_owned();

        if let Err(_error) = std::panic::catch_unwind(|| {
            parse_package(&text).unwrap();
        }) {
            eprintln!("    {BATSU} {filename}");
            errors.push(filename);
        } else {
            eprintln!("    {CHECK} {filename}");
        }
    }
    if errors.len() > 0 {
        panic!("Errors in examples:\n  - {}", errors.join("\n  - "))
    }
}

fn example_files() -> impl Iterator<Item = std::path::PathBuf> {
    let mut results = vec![];
    let examples_dir = std::path::Path::new("..").join("examples");
    let entries = std::fs::read_dir(examples_dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let filename = entry.file_name().to_string_lossy().into_owned();
        if filename.ends_with(".vir") {
            results.push(entry.path())
        }
    }
    results.into_iter()
}
