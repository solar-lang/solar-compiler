// #![feature(string_leak)]
mod compilation;
pub mod id;
pub mod mir;
mod project;
mod types;
mod util;
mod value;

use project::{read_all_projects, read_modules};

use compilation::CompilerContext;

fn main() {
    // Root directory of the solar code project that we intent to compile
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    // config file for solar code
    let project_info = read_all_projects(&fsroot).expect("read in solar project and dependencies");

    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext::with_default_io(&project_info, modules);

    let f_main = ctx.find_target_main().expect("find main function");

    let function_id = ctx.compile_symbol(f_main, &[]).expect("compile code");

    eprintln!("\n{function_id:#?}");

    for (k, _i, v) in ctx.functions.read().unwrap().iter() {
        let k = k.0 .0.join(".") + &format!(".{}", k.0 .1);
        eprintln!("{}:\n{:#?}\n", k, v);
    }
}
