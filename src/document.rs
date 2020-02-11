use crate::elements::Element;
use std::fs;

pub struct Document {
    elements: Vec<Element>
}

impl Document {
    pub fn new(text: &str) -> Self {
        Document {
            elements: Element::parse_elements(text)
        }
    }

   pub fn set_variable(&mut self, key: &str, value: &str) -> usize {
        let compare = |element: &Element| {
            if let Element::Variable(varval) = element {
                if varval == key {
                    return true;
                }
            }

            false
        };

        let mut variables_processed = 0;
        loop {
            if let Some(index) = self.elements.iter().position(compare) {
                self.elements.remove(index);
                self.elements.insert(index, Element::Text(value.to_owned()));
                variables_processed += 1;
            } else {
                return variables_processed;
            }
        }
    }

    pub fn process_includes(&mut self) -> Result<usize, ()>{
        let compare = |element: &Element| {
            if let Element::Include(filename) = element {
                true
            } else {
                false
            }
        };

        let mut includes_processed = 0;
        loop {
            if let Some(index) = self.elements.iter().position(compare) {
                let filename = if let Element::Include(edata) = self.elements.get(index).unwrap() {
                    edata
                } else {
                    panic!("How did bsearch find this?");
                };

                //TODO: Handle errors correctly
                let contents = fs::read_to_string(filename).unwrap();
                self.elements.remove(index);

                let include_elements = Element::parse_elements(&contents);
                for element in include_elements.into_iter().rev() {
                    self.elements.insert(index, element);
                }

                includes_processed += 1;
            } else {
                return Ok(includes_processed);
            }
        }
    }

    /*pub fn get_pattern(&self, name: &str) -> Result<Error, Document> {
        /*
        Find the PatternStart with the name `name`
        Work until the first PatternEnd, tempaltes can't overlap
        Return these elements as a new document
        */
    }

    pub fn set_pattern(&mut self, name: &str, pattern: Document) -> Result<Error, ()> {
        /*
        Find the pattern with this name
        Insert all elements from `pattern` document before it
        */
    }

    pub fn as_string() -> String {
        /*
        Maybe this would be better as like IntoString
        make sure to remove anything that isn't Text
        also remove everything in between and including patterns
        cat all the Element::Text-s together
        */
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include() {
        let test_str = "Test{~ $var ~}{~ @include test/testdoc ~}tesT";
        let cmp_before_vec =[
            Element::Text(String::from("Test")),
            Element::Variable(String::from("var")),
            Element::Include(String::from("test/testdoc")),
            Element::Text(String::from("tesT"))
        ];
        let cmp_after_vec =[
            Element::Text(String::from("Test")),
            Element::Variable(String::from("var")),
            Element::Text(String::from("Testdoc!")),
            Element::Variable(String::from("var")),
            Element::Text(String::from("\n")),
            Element::Text(String::from("tesT"))
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_before_vec);

        let includes = doc.process_includes().unwrap();
        assert_eq!(1, includes);
        assert_eq!(doc.elements, cmp_after_vec);
    }

    #[test]
    fn test_nested_include() {
        let test_str = "{~ @include test/testdoc_nested ~}";
        let cmp_before_vec = [
            Element::Include(String::from("test/testdoc_nested"))
        ];
        let cmp_after_vec = [
            Element::Text(String::from("Testdoc!")), // From testdoc
            Element::Variable(String::from("var")), // From testdoc
            Element::Text(String::from("\n")), // From testdoc
            Element::Text(String::from("\n")) // From testdoc_nested
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_before_vec);

        let includes = doc.process_includes().unwrap();
        assert_eq!(2, includes);
        assert_eq!(doc.elements, cmp_after_vec);
    }

    #[test]
    fn test_variables() {
        let test_str = "Test!{~ $foo ~}";
        let cmp_before_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Variable(String::from("foo"))
        ];
        let cmp_after_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar"))
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_before_vec);

        let variables = doc.set_variable("foo", "bar");
        assert_eq!(1, variables);
        assert_eq!(doc.elements, cmp_after_vec);
    }

    #[test]
    fn test_multivariables() {
        let test_str = "Test!{~ $foo ~}{~ $foobar ~}{~ $foo ~}";
        let cmp_before_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Variable(String::from("foo")),
            Element::Variable(String::from("foobar")),
            Element::Variable(String::from("foo"))
        ];
        let cmp_after_foo_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar")),
            Element::Variable(String::from("foobar")),
            Element::Text(String::from("bar"))
        ];
        let cmp_after_foobar_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar")),
            Element::Text(String::from("barfoo")),
            Element::Text(String::from("bar"))
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_before_vec);

        let variables = doc.set_variable("foo", "bar");
        assert_eq!(2, variables);
        assert_eq!(doc.elements, cmp_after_foo_vec);

       let variables = doc.set_variable("foobar", "barfoo");
        assert_eq!(1, variables);
        assert_eq!(doc.elements, cmp_after_foobar_vec);
    }
}

