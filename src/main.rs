use std::panic;

use colorize::AnsiColor;
use rema::Rema;

fn main() {
    let result = catch_unwind_silent(|| {
        Rema::run();
    });

    match result {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(err) => {
            println!("\n");
            if let Some(msg) = err.downcast_ref::<&str>() {
                if msg.contains("OperationInterrupted") {
                    println!("Operation interrupted by user");
                } else {
                    let error_msg = format!("Application error: {}", msg);
                    eprintln!("{}", error_msg.red());
                }
            } else if let Some(msg) = err.downcast_ref::<String>() {
                if msg.contains("OperationInterrupted") {
                    println!("Operation interrupted by user");
                } else {
                    let error_msg = format!("Application error: {}", msg);
                    eprintln!("{}", error_msg.red());
                }
            } else {
                eprintln!("{}", "An unknown error occurred.".red());
            }
            std::process::exit(1);
        }
    }
}

/// Catch panic without printing the error message.
///
/// See [writing test that ensures panic has occurred](https://stackoverflow.com/questions/26469715/how-do-i-write-a-rust-unit-test-that-ensures-that-a-panic-has-occurred)
fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}
