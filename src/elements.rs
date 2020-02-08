#[derive(Debug, PartialEq)]
pub enum Element {
    Text(String),
    Variable(String),
    Include(String),
    Template(String, Vec<Element>),
    TemplateStart(String),
    TemplateEnd
}

impl Element {
    pub fn parse_elements(text: &str) -> Vec<Element> {
        let mut elements: Vec<Element> = vec![];
        let mut text = text;

        loop {
            if text.is_empty() {
                break;
            }

            if let Some((start, end)) = Self::find_next_element(text) {
                if let Some(element) = Self::parse_element(&text[start..end]) {
                    if !text[..start].is_empty() {
                        elements.push(Element::Text(text[..start].to_owned()));
                    }

                    if let Element::TemplateEnd = element {
                        let mut tempvec: Vec<Element> = vec![];

                        loop {
                            //TODO: Handle error
                            let elem = elements.pop().unwrap();

                            if let Element::TemplateStart(name) = elem {
                                tempvec.reverse();
                                elements.push(Element::Template(name, tempvec));
                                break;
                            } else {
                                tempvec.push(elem);
                            }
                        }
                    } else {
                        elements.push(element);
                    }
                } else {
                    if !text[..end].is_empty() {
                        elements.push(Element::Text(text[..end].to_owned()));
                    }
                }

                text = &text[end..];
            } else {
                elements.push(Element::Text(text.to_owned()));
                break;
            }
        }

        elements
    }

    fn parse_element(text: &str) -> Option<Element> {
        if !(text.starts_with("{~ ") && text.ends_with(" ~}")) {
            println!("Doesn't start and end");
            return None;
        }

        let text = &text[3..text.len()-3];
        if text.len() < 1 {
            return None;
        }

        if text.starts_with('@') {
            return Self::parse_command(&text[1..]);
        } else if text.starts_with('$') {
            return Some(Element::Variable(text[1..].to_owned()));
        }

        None
    }

    fn parse_command(text: &str) -> Option<Element> {
        if text.starts_with("include ") {
            return Some(Element::Include(text[8..].to_owned()));
        } else if text.starts_with("template ") {
            return Some(Element::TemplateStart(text[9..].to_owned()));
        } else if text == "end-template" {
            return Some(Element::TemplateEnd);
        }

        None
    }

    fn find_next_element(text: &str) -> Option<(usize, usize)> {
        if let Some(start) = text.find("{~ ") {
            if let Some(end) = &text[start..].find(" ~}") {
                return Some((start, start+end+3));
            }
        }

        None
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
            Element::Variable(String::from("test")),
            Element::Text(String::from("!"))
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }

    #[test]
    fn test_parse_complex() {
        let test_str = "This is a {~ $word ~} string! Did I spell it right? {~ @include dictionary.txt ~}";
        let cmp_vec = vec![
            Element::Text(String::from("This is a ")),
            Element::Variable(String::from("word")),
            Element::Text(String::from(" string! Did I spell it right? ")),
            Element::Include(String::from("dictionary.txt")),
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }

    #[test]
    fn test_parse_template() {
        let test_str = "Template test\n{~ @template listItem ~}\n<li>{~ $text ~}</li>\n{~ @end-template ~}";
        let cmp_vec = vec![
            Element::Text(String::from("Template test\n")),
            Element::Template(String::from("listItem"), vec![
                Element::Text(String::from("\n<li>")),
                Element::Variable(String::from("text")),
                Element::Text(String::from("</li>\n"))
            ])
        ];

        let parsed = Element::parse_elements(&test_str);
        assert_eq!(parsed, cmp_vec);
    }
}
