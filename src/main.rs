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

use crate::mir::eval::EvaluationContext;

fn main() {
    // Root directory of the solar code project that we intent to compile
    let fsroot = std::env::args().nth(1).unwrap_or(".".to_string());
    // config file for solar code
    let project_info = read_all_projects(&fsroot).expect("read solar project and dependencies");

    let modules = read_modules(&project_info).expect("open and parse solar files");

    let ctx = CompilerContext::with_default_io(&project_info, modules);

    let main_symbol_id = ctx.find_target_main().expect("find main function");

    let (main_function_id, _main_ret_type_id) = ctx
        .compile_symbol(main_symbol_id, &[])
        .expect("compile code");
    eprintln!("\n{main_function_id:#?}");

    let mut ctx: EvaluationContext = ctx.into();

    let res = ctx.call(main_function_id, Vec::new());
    dbg!(res);
}
