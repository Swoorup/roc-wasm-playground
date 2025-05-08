use std::io::{self, BufRead, Write};

use crate::repl::entrypoint_from_wasmtime;

/// Process Roc code content and return the result
pub async fn process_roc_content(content: String) -> io::Result<()> {
    // Process input through Roc REPL
    let result = entrypoint_from_wasmtime(content).await;

    // Print the result
    if !result.is_empty() {
        println!("{}", result);
    }

    Ok(())
}

/// Runs an interactive REPL for Roc code.
///
/// Reads lines from stdin, processes them through the Roc REPL, and prints results to stdout.
pub async fn stdin_to_entrypoint() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    // Start REPL loop
    loop {
        // Print prompt
        print!("> ");
        stdout.flush()?;

        // Read a line of input
        let mut input = String::new();
        let bytes_read = reader.read_line(&mut input)?;

        // Exit if EOF
        if bytes_read == 0 {
            break;
        }

        // Check for quit command
        if input.trim() == ":q" {
            break;
        }

        // Check for help command
        if input.trim() == ":help" {
            println!("\n  - ctrl-v + ctrl-j makes a newline");
            println!("  - :q quits");
            println!("  - :help shows this text again\n");
            continue;
        }

        // Process input through Roc REPL
        let result = entrypoint_from_wasmtime(input.clone()).await;

        // Print the result
        if !result.is_empty() {
            println!("{}", result);
        }
    }

    Ok(())
}
