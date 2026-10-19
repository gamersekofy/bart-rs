//! The `bart-rs` command line tool.
//!
//! All the work happens in the library; this binary is a thin adapter over
//! [`bart_rs::cli`].

use bart_rs::Result;

fn main() -> Result<()> {
    // `Result`'s `Termination` impl would print the `Debug` form of the
    // error, which is not what a user wants to read. Print the `Display`
    // form ourselves and exit with a failure status.
    if let Err(error) = bart_rs::cli::run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
    Ok(())
}
