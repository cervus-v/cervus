extern crate wasm_core;
extern crate cvctl;

use std::fs::File;
use std::env;
use std::io::Read;

use wasm_core::trans::config::ModuleConfig;
use wasm_core::hetrans::translate_module;

use cvctl::cwa_trans::Mapper;

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let path = args.next().expect("Path required");
    let mut f = File::open(&path).unwrap();
    let mut code: Vec<u8> = Vec::new();

    let cfg: ModuleConfig = ModuleConfig::default();

    f.read_to_end(&mut code).unwrap();
    let module = wasm_core::trans::translate_module_raw(code.as_slice(), cfg);
    let entry_fn = module.lookup_exported_func("__app_main").expect("Entry function `__cv_main` not found");

    let mut ctx = cvctl::service::ServiceContext::connect().unwrap();

    let result = translate_module(&module, entry_fn, &mut Mapper::new(&ctx));
    eprintln!("Code length: {}", result.len());

    let mut target_args: Vec<String> = Vec::new();
    target_args.push(path.clone());
    target_args.extend(args);

    let arg_refs: Vec<&str> = target_args.iter().map(|v| v.as_str()).collect();

    let ret = ctx.run_code(&result, cvctl::service::Backend::HexagonE, &arg_refs).unwrap();

    println!("Return code: {}", ret);
}
