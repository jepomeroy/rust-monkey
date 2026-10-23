//! Read-eval-print loop. For now it only tokenizes each line and prints the tokens.

use std::io::{self, BufRead, Write};

use crate::{ast::Node, lexer::Lexer, parser::Parser};

const PROMPT: &str = ">> ";

const MONKEY_FACE: &str = r#"
            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'"#;

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
        let lexer = Lexer::new(&buff);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if parser.errors().len() != 0 {
            let _ = writeln!(out, "{}", MONKEY_FACE);
            let _ = writeln!(out, "Woops! We ran into some monkey business here!");
            let _ = writeln!(out, "*** parser errors:");

            for err in parser.errors() {
                let _ = writeln!(out, "\t{}", err);
            }
        }

        let _ = writeln!(out, "{}", program.string());
        let _ = writeln!(out, "");
    }

    Ok(())
}
