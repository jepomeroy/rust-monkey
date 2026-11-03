//! An interpreter for the Monkey programming language, written in Rust.
//!
//! Source text flows through the [`lexer`] into a stream of [`token`]s, which
//! the [`parser`] turns into an [`ast`]. The [`repl`] module ties these
//! together into an interactive prompt.

mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod repl;
#[cfg(test)]
mod test_utils;
mod token;

use std::{
    env,
    io::{self, stdin, stdout},
};

/// Greets the current user (from `$USER`) and starts the REPL on stdin/stdout.
fn main() -> io::Result<()> {
    let user = match env::var("USER") {
        Ok(user) => user,
        Err(_) => "user".to_string(),
    };

    println!("Hello {}, this is the Monkey programming language!", user);
    println!("Feel free to type commands");

    repl::start(stdin().lock(), stdout())
}
