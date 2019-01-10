use std::collections::HashMap;

pub struct KalFn;

pub enum KalVal {
    Boolean(bool),
    Integer(i64),
    String(String),
    Object(HashMap<String, KalVal>),
    List(Vec<KalVal>),
    Function(KalFn),
}

#[derive(PartialEq, Eq, Hash)]
pub enum Token {
    Whitespace,
    Whitespace_Indent,

    Identifier(String),

    Keyword_Function,

    Syntax_String_Quote,
    Syntax_String_Escape,
    Syntax_String_Newline,

    Syntax_Function_Body_Brace_Open,
    Syntax_Function_Body_Brace_Close,

    Syntax_Operator_Add,

    Literal_Boolean_True,
    Literal_Boolean_False,

    Literal_Number(String),
}

pub type TokenMap = HashMap<Token, &'static str>;

fn default_tokens() -> TokenMap {
    let mut tok = HashMap::new();

    use Token::*;

    tok.insert(Keyword_Function, "fn");

    tok.insert(Syntax_Function_Body_Brace_Open, "{");
    tok.insert(Syntax_Function_Body_Brace_Close, "}");
    tok.insert(Syntax_String_Quote, "\"");
    tok.insert(Syntax_String_Escape, "\\");
    tok.insert(Syntax_String_Newline, "n");

    tok.insert(Literal_Boolean_True, "true");
    tok.insert(Literal_Boolean_False, "false");

    tok
}

mod tokeniser {

    use super::Token;
    use super::TokenMap;
    use std::str::Chars;

    enum MatchResult {
        MatchesMany,
        MatchesOne(Token),
        MatchesNone,
    }

    pub struct Tokeniser<'a> {
        tokens: TokenMap,
        chars_iterator: Chars<'a>,
        location: usize,
    }

    impl<'a> Tokeniser<'a> {
        pub fn new(tokens: TokenMap, input: &'a str) -> Self {
            Tokeniser {
                tokens,
                chars_iterator: input.chars(),
                location: 0,
            }
        }

        fn matches_any(&self, prefix: &str) -> MatchResult {
            let mut count = 0;
            let matched_token = None;
            for (token, token_text) in self.tokens.iter() {
                if token_text.starts_with(prefix) {
                    token = count += 1;
                }
            }

            match count {
                0 => MatchResult::MatchesNone,
                1 => MatchResult::MatchesOne(),
                _ => MatchResult::MatchesMany,
            }
        }
    }

    impl<'a> Iterator for Tokeniser<'a> {
        type Item = Option<Token>;

        fn next(&mut self) -> Option<Option<Token>> {
            None
        }
    }

}

mod parser {

    pub use super::{KalFn, KalVal};

    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub struct ParseError(&'static str);

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Error: {}", self.0)
        }
    }

    impl Error for ParseError {}

    pub fn parse(function: String) -> Result<KalFn, ParseError> {
        Ok(KalFn {})
    }
}

fn main() {
    println!("Hello, world!");
}
