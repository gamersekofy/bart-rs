//! The `bart-rs` command line tool.
//!
//! All the work happens in the library; this binary is a thin adapter over
//! [`bart_rs::cli`].

use std::any::Any;
use std::panic::AssertUnwindSafe;

use bart_rs::Result;

fn main() -> Result<()> {
    // A closed output pipe — `bart-rs routes | head` — is not a failure.
    // Rust's runtime ignores `SIGPIPE`, so the next `println!` panics with
    // "failed printing to stdout: Broken pipe" and the process exits 101.
    // Every other Unix tool dies quietly instead, and so does this one.
    silence_broken_pipe_panics();

    let outcome = std::panic::catch_unwind(AssertUnwindSafe(bart_rs::cli::run));

    match outcome {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => {
            // `Result`'s `Termination` impl would print the `Debug` form of
            // the error, which is not what a user wants to read. Print the
            // `Display` form ourselves and exit with a failure status.
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
        Err(payload) if is_broken_pipe(payload.as_ref()) => {
            // The reader went away. Nothing went wrong.
            std::process::exit(0);
        }
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Whether a panic payload was caused by the output pipe being closed.
fn is_broken_pipe(payload: &(dyn Any + Send)) -> bool {
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    message.contains("Broken pipe")
}

/// Stop the default panic hook from printing a broken-pipe panic.
///
/// The hook still runs for every other panic, so genuine bugs are reported
/// as usual.
fn silence_broken_pipe_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !is_broken_pipe(info.payload()) {
            previous(info);
        }
    }));
}
