use std::ffi::CString;

use nix::unistd::execvp;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let bin_path = std::env::current_exe().unwrap();
    let bin_dir = bin_path.parent().unwrap();

    // if you call vir with `$ vir foo`
    // it will look for an executable vir-foo
    let cmd = args.get(1).unwrap();
    let bin = bin_dir.join(format!("vir-{cmd}"));

    let program = CString::new(bin.to_str().unwrap()).unwrap();
    let args: Vec<CString> = args[1..].into_iter().map(|arg| CString::new(arg.as_str()).unwrap()).collect();
    execvp(&program, &args).unwrap();
}
