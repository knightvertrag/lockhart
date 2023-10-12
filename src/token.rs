use phf::phf_map;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokenType {
    IDENT,
    NUM,
    STRING,
    LET,
    FUNCTION,
    IF,
    ELSE,
    FOR,
    WHILE,
    PRINT,
    RETURN,
    TRUE,
    FALSE,
    NIL,
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
    AND,
    OR,
    COMMA,
    SEMICOLON,
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN,
    ILLEGAL,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    pub literal: String,
    pub lineno: usize,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.literal == other.literal
    }
}

impl Eq for Token {}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.literal.hash(state);
    }
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
    "print" => TokenType::PRINT,
    "true" => TokenType::TRUE,
    "false" => TokenType::FALSE,
    "return" => TokenType::RETURN,
    "if" => TokenType::IF,
    "else" => TokenType::ELSE,
    "and" => TokenType::AND,
    "or" => TokenType::OR,
    "for" => TokenType::FOR,
    "while" => TokenType::WHILE,
    "nil" => TokenType::NIL,
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
