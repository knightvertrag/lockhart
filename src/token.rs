use phf::phf_map;

// pub type TokenType = &'static str;
#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    IDENT(Ident),
    NUM(Num),
    KEYWORDS(Keywords),
    OPERATORS(Operators),
    DELIMITERS(Delimiters),
    ILLEGAL
}

#[derive(PartialEq, Clone, Debug)]
pub enum Ident {
    IDENT,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Num {
    NUM,
}
#[derive(PartialEq, Clone, Debug)]
pub enum Delimiters {
    COMMA,
    SEMICOLON,
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN
}
#[derive(PartialEq, Clone, Debug)]
pub enum Keywords {
    LET,
    FUNCTION,
    IF,
    ELSE,
    RETURN,
    TRUE,
    FALSE
}
#[derive(PartialEq, Clone, Debug)]
pub enum Operators {
    ASSIGN,
    NOT,
    GT,
    LT,
    GEQ,
    LEQ,
    EQ,
    NEQ
}

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
        TokenType::IDENT(Ident::IDENT)
    }
}

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "let" => TokenType::KEYWORDS(Keywords::LET),
    "fn" => TokenType::KEYWORDS(Keywords::FUNCTION),
    "true" => TokenType::KEYWORDS(Keywords::TRUE),
    "false" => TokenType::KEYWORDS(Keywords::FALSE),
    "return" => TokenType::KEYWORDS(Keywords::RETURN),
    "if" => TokenType::KEYWORDS(Keywords::IF),
    "else" => TokenType::KEYWORDS(Keywords::ELSE)
};

pub static OPERATORS: phf::Map<&'static str, TokenType> = phf_map! {
    "=" => TokenType::OPERATORS(Operators::ASSIGN),
    "!" => TokenType::OPERATORS(Operators::NOT),
    "<" => TokenType::OPERATORS(Operators::LT),
    ">" => TokenType::OPERATORS(Operators::GT),
    "==" => TokenType::OPERATORS(Operators::EQ),
    "!=" => TokenType::OPERATORS(Operators::NEQ),
    ">=" => TokenType::OPERATORS(Operators::GEQ),
    "<=" => TokenType::OPERATORS(Operators::LEQ)
};

pub static DELIMITERS: phf::Map<&'static str, TokenType> = phf_map! {
    "{" => TokenType::DELIMITERS(Delimiters::LBRACE),
    "}" => TokenType::DELIMITERS(Delimiters::RBRACE),
    "(" => TokenType::DELIMITERS(Delimiters::LPAREN),
    ")" => TokenType::DELIMITERS(Delimiters::RPAREN),
    "," => TokenType::DELIMITERS(Delimiters::COMMA),
    ";" => TokenType::DELIMITERS(Delimiters::SEMICOLON)
};