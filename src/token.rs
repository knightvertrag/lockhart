use phf::phf_map;

pub type TokenType = &'static str;

#[derive(PartialEq, Debug)]
pub struct Token {
    pub type_: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(type_: TokenType, literal: String) -> Token {
        Token { type_, literal }
    }

    pub fn check_keyword(ident: &String) -> TokenType {
        if let Some(key) = KEYWORDS.get(ident).cloned() {
            return key;
        }
        IDENT
    }
}
// identifiers + literals
pub const IDENT: &'static str = "IDENT"; //foobar, x, y.....
pub const NUM: &'static str = "NUM"; // 123456....

// Operators
pub const ASSIGN: &'static str = "=";
pub const PLUS: &'static str = "+";

// Delimiters
pub const COMMA: &'static str = ",";
pub const SEMICOLON: &'static str = ";";

pub const LPAREN: &'static str = "(";
pub const RPAREN: &'static str = ")";
pub const LBRACE: &'static str = "{";
pub const RBRACE: &'static str = "}";

// Keywords
pub const FUNCTION: &'static str = "FUNCTION";
pub const LET: &'static str = "LET";

// SPECIAL
pub const EOF: &'static str = "EOF";
pub const ILLEGAL: &'static str = "ILLEGAL";

// TODO: Probably change this entire thing to an enum
// pub enum Keywords {
//     Let(&'static str),
//     Fn(&'static str)
//     .....
// }

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "let" => LET,
    "fn" => FUNCTION
};
