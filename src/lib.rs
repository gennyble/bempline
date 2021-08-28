use std::{
    collections::HashMap,
    iter::Peekable,
    str::{Chars, FromStr},
};

#[derive(Clone, Debug)]
pub struct Document {
    tokens: Vec<Token>,
    variables: HashMap<String, String>,
}

impl FromStr for Document {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let mut tokens = vec![];

        let mut current = String::new();
        let mut chars = raw.chars().peekable();
        loop {
            match chars.next() {
                // Escapes
                Some('\\') => match chars.next() {
                    // Only esccape the opening brace
                    Some('{') => current.push('{'),
                    // Keep \ if { is not next
                    Some(ch) => {
                        current.push('\\');
                        current.push(ch);
                    }
                    // leave it up to the other None handler
                    None => (),
                },
                Some('{') => {
                    let key = take_while_chars(&mut chars, |ch| ch.is_alphanumeric() || *ch == '_');

                    match chars.peek() {
                        // Variable is valid!
                        Some('}') => {
                            if !current.is_empty() {
                                tokens.push(Token::Text(current.clone()));
                                current.clear();
                            }

                            tokens.push(Token::Variable(key));
                            chars.next(); // throw away the }
                        }
                        // Variable was not valid, we have to recover
                        _ => {
                            current.push('{');
                            current.push_str(&key);
                        }
                    }
                }
                Some(ch) => current.push(ch),
                None => {
                    if !current.is_empty() {
                        tokens.push(Token::Text(current));
                    }
                    break;
                }
            }
        }

        Ok(Self {
            tokens,
            variables: HashMap::new(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Text(String),
    Variable(String),
}

fn take_while_chars(iter: &mut Peekable<Chars>, func: impl Fn(&char) -> bool) -> String {
    let mut ret = String::new();

    loop {
        match iter.peek() {
            Some(ch) if func(ch) => ret.push(iter.next().unwrap()),
            _ => break,
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    // Parsing related tests

    #[test]
    fn no_text() {
        let doc = Document::from_str("").unwrap();
        assert_eq!(doc.tokens, vec![]);
    }

    #[test]
    fn only_text() {
        let doc = Document::from_str("Nothing but text").unwrap();
        assert_eq!(
            doc.tokens,
            vec![Token::Text(String::from("Nothing but text"))]
        );
    }

    #[test]
    fn escaped_bracket() {
        let doc = Document::from_str("escape this: \\{, but not this \\n").unwrap();
        assert_eq!(
            doc.tokens,
            vec![Token::Text(String::from(
                "escape this: {, but not this \\n"
            ))]
        );
    }

    #[test]
    fn only_variable() {
        let doc = Document::from_str("{variable}").unwrap();
        assert_eq!(doc.tokens, vec![Token::Variable(String::from("variable"))]);
    }

    #[test]
    fn sandwhiched_variable() {
        let doc = Document::from_str("Hello {name}, how are you?").unwrap();
        assert_eq!(
            doc.tokens,
            vec![
                Token::Text(String::from("Hello ")),
                Token::Variable(String::from("name")),
                Token::Text(String::from(", how are you?"))
            ]
        );
    }

    #[test]
    fn ends_variable() {
        let doc = Document::from_str("Hello {name}").unwrap();
        assert_eq!(
            doc.tokens,
            vec![
                Token::Text(String::from("Hello ")),
                Token::Variable(String::from("name"))
            ]
        );
    }

    #[test]
    fn starts_variable() {
        let doc = Document::from_str("{name}, hello!").unwrap();
        assert_eq!(
            doc.tokens,
            vec![
                Token::Variable(String::from("name")),
                Token::Text(String::from(", hello!"))
            ]
        );
    }

    #[test]
    fn multivariable() {
        let doc = Document::from_str("The weather is {weather} in {location} today.").unwrap();
        assert_eq!(
            doc.tokens,
            vec![
                Token::Text(String::from("The weather is ")),
                Token::Variable(String::from("weather")),
                Token::Text(String::from(" in ")),
                Token::Variable(String::from("location")),
                Token::Text(String::from(" today."))
            ]
        );
    }
}
