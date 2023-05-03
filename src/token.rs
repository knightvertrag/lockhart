use phf::phf_map;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    IDENT,
    NUM,
    LITERAL,
    LET,
    FUNCTION,
    IF,
    ELSE,
    RETURN,
    TRUE,
    FALSE,
    ASSIGN,
    NOT,
    GT,
    LT,
    GEQ,
    LEQ,
    EQ,
    NEQ,
    PLUS,
    MINUS,
    MUL,
    DIV,
    COMMA,
    SEMICOLON,
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN,
    ILLEGAL,
    EOF,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    pub literal: String,
    pub lineno: usize,
}

impl Token {
    pub fn new(type_: TokenType, literal: String, lineno: usize) -> Token {
        Token {
            type_,
            literal,
            lineno,
        }
    }

    pub fn new_def() -> Token {
        Token {
            type_: TokenType::ILLEGAL,
            literal: "".to_string(),
            lineno: 0,
        }
    }

    pub fn check_keyword(ident: &String) -> TokenType {
        if let Some(key) = KEYWORDS.get(ident).cloned() {
            return key;
        }
        TokenType::IDENT
    }
}

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "let" => TokenType::LET,
    "fn" => TokenType::FUNCTION,
    "true" => TokenType::TRUE,
    "false" => TokenType::FALSE,
    "return" => TokenType::RETURN,
    "if" => TokenType::IF,
    "else" => TokenType::ELSE
};

pub static OPERATORS: phf::Map<&'static str, TokenType> = phf_map! {
    "=" => TokenType::ASSIGN,
    "!" => TokenType::NOT,
    "<" => TokenType::LT,
    ">" => TokenType::GT,
    "==" => TokenType::EQ,
    "!=" => TokenType::NEQ,
    ">=" => TokenType::GEQ,
    "<=" => TokenType::LEQ,
    "+" => TokenType::PLUS,
    "-" => TokenType::MINUS,
    "*" => TokenType::MUL,
    "/" => TokenType::DIV,
};

pub static DELIMITERS: phf::Map<&'static str, TokenType> = phf_map! {
    "{" => TokenType::LBRACE,
    "}" => TokenType::RBRACE,
    "(" => TokenType::LPAREN,
    ")" => TokenType::RPAREN,
    "," => TokenType::COMMA,
    ";" => TokenType::SEMICOLON,
};
