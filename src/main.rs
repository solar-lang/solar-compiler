// #![feature(string_leak)]
mod compilation;
pub mod id;
pub mod mir;
mod project;
mod types;
mod util;

use project::{read_all_projects, read_modules};

use compilation::CompilerContext;

use crate::mir::eval::EvaluationContext;

fn main() {
    // Root directory of the solar code project that we intent to compile
    let config = test_config();
    // config file for solar code
    let project_info = read_all_projects(&config).expect("read solar project and dependencies");

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

pub struct Config {
    pub project_root: String,
    pub solarpath: String,
}

impl Config {
    fn load() -> Self {
    let project_root = std::env::args().nth(1).unwrap_or(".".to_string());
        Config { project_root, solarpath: Self::get_solar_path() }
    }

    fn get_solar_path() -> String {
        let solar_path = std::env::var("SOLAR_PATH").unwrap_or("~/.solar/".to_string());
        let home_path = std::env::var("HOME").expect("get home path env variable");
        let solar_path: String = solar_path.replace('~', &home_path);


        solar_path
    }
}

fn test_config() -> Config {
    Config {
        project_root: "./samples/4".to_string(),
        solarpath: "./solarpath".to_string(),
    }
}