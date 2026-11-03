//! Read-eval-print loop.
//!
//! Input is collected line by line into a buffer until every `(`, `{` and `[`
//! has been closed, so multi-line code (typed or pasted) is evaluated as one
//! unit.

use std::io::{self, BufRead, Write};

use crate::{
    ast::Node::ProgramNode,
    evaluator::evaluator::eval,
    lexer::Lexer,
    object::{Env, Environment, Object, ObjectType},
    parser::Parser,
    token::TokenType,
};

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
/// Lines are buffered until all brackets are balanced, then the buffer is
/// parsed and evaluated. Exits at end of input (Ctrl-D).
pub(crate) fn start<R: BufRead, W: Write>(input: R, mut out: W) -> io::Result<()> {
    let env = Environment::new();
    let mut lines = input.lines();
    let mut buff = String::new();

    loop {
        // Only prompt at the start of a statement.
        if buff.is_empty() {
            write!(out, "{}", PROMPT)?;
            out.flush()?;
        }

        let Some(line) = lines.next() else {
            // End of input: run anything still buffered, then quit.
            if !buff.trim().is_empty() {
                writeln!(out)?;
                run(&buff, &env, &mut out)?;
            }
            return Ok(());
        };

        buff.push_str(&line?);
        buff.push('\n');

        if buff.trim().is_empty() {
            buff.clear();
            continue;
        }

        if bracket_depth(&buff) > 0 {
            continue;
        }

        run(&buff, &env, &mut out)?;
        buff.clear();
    }
}

/// Parses and evaluates `source`, printing either the parser errors or the result.
fn run<W: Write>(source: &str, env: &Env, out: &mut W) -> io::Result<()> {
    let mut parser = Parser::new(Lexer::new(source));
    let program = parser.parse_program();

    if !parser.errors().is_empty() {
        return print_parser_errors(out, parser.errors());
    }

    let evaluated = eval(&ProgramNode(program), env);
    if evaluated != ObjectType::Null {
        writeln!(out, "{}", evaluated.inspect())?;
    }

    Ok(())
}

/// Returns how many `(`, `{` and `[` in `source` are still unclosed.
///
/// Uses the lexer so brackets inside string literals aren't counted. A
/// negative result means there are extra closing brackets; the parser will
/// report that.
fn bracket_depth(source: &str) -> i32 {
    let mut lexer = Lexer::new(source);
    let mut depth = 0;

    loop {
        match lexer.next_token().token {
            TokenType::LPAREN | TokenType::LBRACE | TokenType::LBRACKET => depth += 1,
            TokenType::RPAREN | TokenType::RBRACE | TokenType::RBRACKET => depth -= 1,
            TokenType::EOF => return depth,
            _ => {}
        }
    }
}

fn print_parser_errors<W: Write>(out: &mut W, errors: &[String]) -> io::Result<()> {
    writeln!(out, "{}", MONKEY_FACE)?;
    writeln!(out, "Woops! We ran into some monkey business here!")?;
    writeln!(out, "*** parser errors:")?;

    for err in errors {
        writeln!(out, "\t{}", err)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_repl(input: &str) -> String {
        let mut out = Vec::new();
        start(input.as_bytes(), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn bracket_depth_counts_open_brackets() {
        assert_eq!(bracket_depth("let f = fn(x) {"), 1);
        assert_eq!(bracket_depth("let a = [1, (2"), 2);
        assert_eq!(bracket_depth("fn(x) { x }"), 0);
        assert_eq!(bracket_depth("}"), -1);
    }

    #[test]
    fn bracket_depth_ignores_brackets_in_strings() {
        assert_eq!(bracket_depth(r#"let s = "{[(";"#), 0);
    }

    #[test]
    fn single_line_input() {
        assert_eq!(run_repl("1 + 2\n"), ">> 3\n>> ");
    }

    #[test]
    fn multi_line_input_is_buffered() {
        let input = "let add = fn(a, b) {\n  a + b\n};\nadd(2, 3)\n";
        // `let` evaluates to the bound value, so the function is echoed once
        // the buffered definition completes.
        assert_eq!(run_repl(input), ">> fn(a, b) {\n(a + b)\n}\n>> 5\n>> ");
    }

    #[test]
    fn pasted_map_function_with_blank_line() {
        let input = r#"
            let map = fn(arr, f) {
              let iter = fn(arr, accumulated) {
                if (len(arr) == 0) {
                  accumulated
                } else {
                  iter(rest(arr), push(accumulated, f(first(arr))));
                }
              };
            
              iter(arr, []);
            };
            map([1, 2, 3], fn(x) { x * 2 })"#;
        let out = run_repl(input);
        assert!(out.contains("[2, 4, 6]"), "unexpected output: {out}");
        assert!(!out.contains("parser errors"), "unexpected output: {out}");
    }

    #[test]
    fn parser_errors_are_reported() {
        assert!(run_repl("let = 5\n").contains("parser errors"));
    }
}
