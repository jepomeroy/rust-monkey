//! Read-eval-print loop. For now it only tokenizes each line and prints the tokens.

use std::io::{self, BufRead, Write};

use crate::{lexer::Lexer, token::TokenType};

const PROMPT: &str = ">> ";

/// Runs the REPL, reading lines from `input` and writing results to `out`.
///
/// Exits when an empty line is entered.
pub(crate) fn start<R: BufRead, W: Write>(mut input: R, mut out: W) -> io::Result<()> {
    loop {
        let mut buff = String::new();
        write!(out, "{}", PROMPT)?;
        out.flush()?;

        let count = input.read_line(&mut buff)?;

        // Just the '\n' char
        if count == 1 {
            break;
        }

        // Print every token on the line until the lexer reaches the end.
        let mut lexer = Lexer::new(buff);
        let mut token = lexer.next_token();
        while token.token != TokenType::EOF {
            writeln!(out, "{:?}", token)?;
            token = lexer.next_token();
        }
    }

    Ok(())
}
