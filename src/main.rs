mod lexer;
mod token;

use token::Token;

fn main() {
    let t = Token {
        token: token::TokenType::ILLEGAL,
        literal: "bar".to_string(),
    };

    println!("Hello, {:?} world!", t);
}
