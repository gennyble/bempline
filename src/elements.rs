use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Element {
    Text(String),
    Variable(String, bool),
    Include(String, bool),
    Pattern(String, Vec<Element>),
    PatternStart(String),
    PatternEnd,
}

impl Element {
    pub fn parse_elements(text: &str) -> Vec<Element> {
        let mut elements: Vec<Element> = vec![];
        let mut text = text;

        loop {
            if text.is_empty() {
                break;
            }

            let (start, end) = match Self::find_next_element(text) {
                Some((start, end)) => (start, end),
                None => {
                    elements.push(Element::Text(text.to_owned()));
                    break;
                }
            };

            let element = match Self::parse_element(&text[start..end]) {
                Some(element) => element,
                None => {
                    if !text[..end].is_empty() {
                        elements.push(Element::Text(text[..end].to_owned()));
                    }

                    break;
                }
            };

            if !text[..start].is_empty() {
                elements.push(Element::Text(text[..start].to_owned()));
            }

            if let Element::PatternEnd = element {
                let mut tempvec: Vec<Element> = vec![];

                loop {
                    //TODO: Handle error caused by a lack of Element::PatternStart
                    let elem = elements.pop().unwrap();

                    if let Element::PatternStart(name) = elem {
                        tempvec.reverse();
                        elements.push(Element::Pattern(name, tempvec));
                        break;
                    } else {
                        tempvec.push(elem);
                    }
                }
            } else {
                elements.push(element);
            }

            text = &text[end..];
        }

        elements
    }

    fn parse_element(text: &str) -> Option<Element> {
        // All elements much start {~ and end ~}
        if !(text.starts_with("{~ ") && text.ends_with(" ~}")) {
            return None;
        }

        // Remove the start/end delimiters
        let text = &text[3..text.len() - 3];
        if text.len() < 1 {
            return None;
        }

        // Identify commands and variables
        if text.starts_with('@') {
            return Self::parse_command(&text[1..]);
        } else if text.starts_with('$') {
            let mut var_name = text[1..].to_owned();

            let required_flag = if var_name.ends_with("!") {
                var_name.pop();
                true
            } else if var_name.ends_with("?") {
                var_name.pop();
                false
            } else {
                false
            };

            return Some(Element::Variable(var_name, required_flag));
        }

        None
    }

    fn parse_command(text: &str) -> Option<Element> {
        let cmd_include = "include ";
        let cmd_include_optional = "include? ";
        let cmd_include_require = "include! ";
        let cmd_pattern_start = "pattern ";
        let cmd_pattern_end = "end-pattern";

        if text.starts_with(cmd_include) {
            return Some(Element::Include(text[cmd_include.len()..].to_owned(), true));
        } else if text.starts_with(cmd_include_optional) {
            return Some(Element::Include(text[cmd_include_optional.len()..].to_owned(), false));
        } else if text.starts_with(cmd_include_require) {
            return Some(Element::Include(text[cmd_include_require.len()..].to_owned(), true));
        } else if text.starts_with(cmd_pattern_start) {
            return Some(Element::PatternStart(text[cmd_pattern_start.len()..].to_owned()));
        } else if text == cmd_pattern_end {
            return Some(Element::PatternEnd);
        }

        None
    }

    fn find_next_element(text: &str) -> Option<(usize, usize)> {
        if let Some(start) = text.find("{~ ") {
            if let Some(end) = &text[start..].find(" ~}") {
                return Some((start, start + end + 3));
            }
        }

        None
    }

    pub fn string(self) -> Result<String, Error> {
        match self {
            Element::Text(text) => Ok(text),
            Element::Variable(name, requried) if requried == true => {
                return Err(Error::UnusedRequire(format!("Required variable with name {}", name)))
            },
            Element::Include(path, requried) if requried == true => {
                return Err(Error::UnusedRequire(format!("Required @include with path {}", path)))
            },
            _ => Ok(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let test_str = "Test{~ $test ~}!";
        let cmp_vec = vec![
            Element::Text(String::from("Test")),
            Element::Variable(String::from("test"), false),
            Element::Text(String::from("!")),
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }

    #[test]
    fn test_parse_complex() {
        let test_str =
            "This is a {~ $word ~} string! Did I spell it right? {~ @include dictionary.txt ~}";
        let cmp_vec = vec![
            Element::Text(String::from("This is a ")),
            Element::Variable(String::from("word"), false),
            Element::Text(String::from(" string! Did I spell it right? ")),
            Element::Include(String::from("dictionary.txt"), true),
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }

    #[test]
    fn test_parse_pattern() {
        let test_str =
            "Pattern test\n{~ @pattern listItem ~}\n<li>{~ $text ~}</li>\n{~ @end-pattern ~}";
        let cmp_vec = vec![
            Element::Text(String::from("Pattern test\n")),
            Element::Pattern(
                String::from("listItem"),
                vec![
                    Element::Text(String::from("\n<li>")),
                    Element::Variable(String::from("text"), false),
                    Element::Text(String::from("</li>\n")),
                ],
            ),
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }

    #[test]
    fn test_to_string() {
        let text = Element::Text(String::from("TextTest"));
        let variable_opt = Element::Variable(String::from("VariableTest"), false);
        let variable_req = Element::Variable(String::from("VariableTest"), true);
        let include_opt = Element::Include(String::from("IncludeTest"), false);
        let include_req = Element::Include(String::from("IncludeTest"), true);
        let pattern = Element::Pattern(String::from("PatternTest"), vec![]);

        assert_eq!(text.string().unwrap(), "TextTest");
        assert_eq!(variable_opt.string().unwrap(), "");
        variable_req.string().unwrap_err();
        assert_eq!(include_opt.string().unwrap(), "");
        include_req.string().unwrap_err();
        assert_eq!(pattern.string().unwrap(), "");
    }

    #[test]
    fn test_variable_optional() {
        let test_str = "{~ $foo? ~}{~ $bar! ~}{~ $foobar ~}";
        let variable_opt = Element::Variable(String::from("foo"), false);
        let variable_req = Element::Variable(String::from("bar"), true);
        let variable_default_opt = Element::Variable(String::from("foobar"), false);

        let mut parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed.pop(), Some(variable_default_opt));
        assert_eq!(parsed.pop(), Some(variable_req));
        assert_eq!(parsed.pop(), Some(variable_opt));
    }

    #[test]
    fn test_include_optional() {
        let test_str = "{~ @include? foo ~}{~ @include! bar ~}{~ @include foobar ~}";
        let include_opt = Element::Include(String::from("foo"), false);
        let include_req = Element::Include(String::from("bar"), true);
        let include_default_opt = Element::Include(String::from("foobar"), true);

        let mut parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed.pop(), Some(include_default_opt));
        assert_eq!(parsed.pop(), Some(include_req));
        assert_eq!(parsed.pop(), Some(include_opt));
    }
}
