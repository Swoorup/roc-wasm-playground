use anyhow::anyhow;
use wasmtime::{Engine, Instance, Memory, Module, Result, Store};

// We'll use this struct to hold the Wasmtime resources
pub struct WasmtimeApp {
    store: Store<()>,
    instance: Instance,
    memory: Memory,
    wrapper_func: wasmtime::TypedFunc<(), i32>,
    result_addr: Option<usize>,
}

impl WasmtimeApp {
    // Create a new Wasmtime instance from WebAssembly binary
    pub fn new(wasm_module_bytes: &[u8]) -> Result<Self> {
        // Create the Wasmtime engine
        let engine = Engine::default();

        // Compile the module
        let module = Module::new(&engine, wasm_module_bytes)?;

        // Create a store which holds the instantiated modules
        let mut store = Store::new(&engine, ());

        // Define the panic handler function to be imported by the module
        let send_panic_msg_to_js = wasmtime::Func::wrap(
            &mut store,
            |mut caller: wasmtime::Caller<'_, ()>, ptr: i32, panic_tag: i32| {
                // Handle Roc panics - more robust implementation
                match panic_tag {
                    0 => println!("Roc failed with message at ptr: {}", ptr),
                    1 => println!("User crash with message at ptr: {}", ptr),
                    _ => println!("Unknown panic tag: {}, ptr: {}", panic_tag, ptr),
                }
                Ok(())
            },
        );

        // Create import objects
        let imports = [wasmtime::Extern::Func(send_panic_msg_to_js)];

        // Instantiate the module with imports
        let instance = Instance::new(&mut store, &module, &imports)?;

        // Get the memory export
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("Failed to find memory export"))?;

        // Get the wrapper function
        let wrapper_func = instance.get_typed_func::<(), i32>(&mut store, "wrapper")?;

        Ok(Self {
            store,
            instance,
            memory,
            wrapper_func,
            result_addr: None,
        })
    }

    pub fn wasmtime_run_app(&mut self) -> usize {
        // Call the wrapper function
        match self.wrapper_func.call(&mut self.store, ()) {
            Ok(result) => {
                // Store the result address
                let result_addr = result as usize;
                self.result_addr = Some(result_addr);
            }
            Err(err) => {
                // Log the error but continue execution
                eprintln!("Error calling wrapper function: {}", err);
                self.result_addr = None;
            }
        }

        // Return the memory size - this will always succeed even if the wrapper function failed
        let memory_size = self.memory.data_size(&self.store);
        memory_size
    }

    // Replaces js_get_result_and_memory
    pub fn wasmtime_get_result_and_memory(&mut self, buffer: &mut [u8]) -> usize {
        // // Get the app's memory data
        let memory_data = self.memory.data(&self.store);

        // Copy the memory to the provided buffer
        let copy_size = std::cmp::min(buffer.len(), memory_data.len());
        buffer[..copy_size].copy_from_slice(&memory_data[..copy_size]);

        // Return the result address (default to 0 if not set)
        self.result_addr.unwrap_or(0)
    }
}

// Replaces js_create_app
pub async fn wasmtime_create_app(wasm_module_bytes: &[u8]) -> Result<WasmtimeApp> {
    let app = WasmtimeApp::new(wasm_module_bytes)?;
    Ok(app)
}
