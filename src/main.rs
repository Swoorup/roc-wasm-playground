//! Roc playground that supports stdin/stdout mode and file reading mode

mod repl;
mod stdin_runner;
mod wasm_runner;
use std::{env, fs};
use tokio;
use wasmtime::*;

pub enum PluginSource<'a> {
    FromWasmFile(&'a str),
    FromWasmBuffer(&'a [u8]),
    FromRocSourceFile(&'a str),
    FromRocSourceBuffer(&'a [u8]),
}

impl PluginSource<'_> {
    fn load_module(&self, engine: &Engine) -> Result<Module> {
        let module = match self {
            PluginSource::FromWasmFile(path) => Module::from_file(engine, path)?,
            PluginSource::FromWasmBuffer(items) => todo!(),
            PluginSource::FromRocSourceFile(path) => todo!(),
            PluginSource::FromRocSourceBuffer(items) => todo!(),
        };

        Ok(module)
    }
}

struct MyState {
    name: String,
    count: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        // Check for stdin mode
        if args[1] == "--stdin" {
            // Run in stdin mode
            if let Err(e) = stdin_runner::stdin_to_entrypoint().await {
                eprintln!("Error processing stdin: {}", e);
                std::process::exit(1);
            }
            return Ok(());
        }
        
        // Check for file mode
        if args[1] == "--file" && args.len() > 2 {
            let file_path = &args[2];
            // Read the file content
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    if let Err(e) = stdin_runner::process_roc_content(content).await {
                        eprintln!("Error processing file {}: {}", file_path, e);
                        std::process::exit(1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error reading file {}: {}", file_path, e);
                    std::process::exit(1);
                }
            }
        }
        
        // Show usage if invalid args
        if args[1] == "--help" || args[1] == "-h" {
            print_usage();
            return Ok(());
        }
    }

    // Otherwise run the default example
    run_default_example()
}

fn print_usage() {
    println!("Usage: cargo run [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --stdin              Run in interactive REPL mode");
    println!("  --file <FILE_PATH>   Process Roc code from the specified file");
    println!("  --help, -h           Show this help message");
    println!();
    println!("Without any options, the program runs a simple WebAssembly example.");
}

/// Run the original wasm example
fn run_default_example() -> Result<()> {
    const HELLO_WAT: &'static str = "examples/hello.wat";

    // First the wasm module needs to be compiled. This is done with a global
    // "compilation environment" within an `Engine`. Note that engines can be
    // further configured through `Config` if desired instead of using the
    // default like this is here.
    println!("Compiling module...");

    let engine = Engine::default();
    let module = PluginSource::FromWasmFile(HELLO_WAT).load_module(&engine)?;

    // After a module is compiled we create a `Store` which will contain
    // instantiated modules and other items like host functions. A Store
    // contains an arbitrary piece of host information, and we use `MyState`
    // here.
    println!("Initializing...");
    let mut store = Store::new(
        &engine,
        MyState {
            name: "hello, world!".to_string(),
            count: 0,
        },
    );

    // Our wasm module we'll be instantiating requires one imported function.
    // the function takes no parameters and returns no results. We create a host
    // implementation of that function here, and the `caller` parameter here is
    // used to get access to our original `MyState` value.
    println!("Creating callback...");
    let hello_func = Func::wrap(&mut store, |mut caller: Caller<'_, MyState>| {
        println!("Calling back...");
        println!("> {}", caller.data().name);
        caller.data_mut().count += 1;
    });

    // Once we've got that all set up we can then move to the instantiation
    // phase, pairing together a compiled module as well as a set of imports.
    // Note that this is where the wasm `start` function, if any, would run.
    println!("Instantiating module...");
    let imports = [hello_func.into()];
    let instance = Instance::new(&mut store, &module, &imports)?;

    // Next we poke around a bit to extract the `run` function from the module.
    println!("Extracting export...");
    let run = instance.get_typed_func::<(), ()>(&mut store, "run")?;

    // And last but not least we can call it!
    println!("Calling export...");
    run.call(&mut store, ())?;

    println!("Done.");
    
    // Add a pause to make output more visible
    println!("\nPress Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    
    Ok(())
}
