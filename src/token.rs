type TokenType = String;

pub struct Token {
    pub _type: TokenType,
    pub literal: String,
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



