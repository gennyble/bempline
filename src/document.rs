use crate::elements::Element;
use crate::error::Error;
use std::fs;

#[derive(Clone)]
pub struct Document {
    elements: Vec<Element>,
}

impl Document {
    pub fn new(text: &str) -> Self {
        Document {
            elements: Element::parse_elements(text),
        }
    }

    /// Returns a usize indicating how many variables were replaced
    ///
    /// Replace every instance of variable `key` with text `value`
    ///
    /// # Example
    /// ```rust
    /// use bempline::Document;
    ///
    /// let mut doc = Document::new("Hello, {~ $noun ~}!");
    /// let num_replaced = doc.set_variable("noun", "World");
    ///
    /// assert_eq!(num_replaced, 1);
    /// assert_eq!(doc.as_string(), "Hello, World!");
    /// ```
    pub fn set_variable(&mut self, key: &str, value: &str) -> usize {
        let compare = |element: &Element| match element {
            Element::Variable(vkey) if vkey == key => true,
            _ => false
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

    /// Returns a usize indicating the number of includes that were processed
    /// successfully.
    ///
    /// Replaces every include command in the document with the content of the
    /// file it references.
    ///
    /// # Errors
    /// Returns a [`Error`] if a file referenced by one of the include
    /// statements is unable to be opened for reading.
    ///
    /// [`Error`]: enum.Error.html
    pub fn process_includes(&mut self) -> Result<usize, Error> {
        let compare = |element: &Element| match element {
            Element::Include(_) => true,
            _ => false
        };

        let mut includes_processed = 0;
        loop {
            if let Some(index) = self.elements.iter().position(compare) {
                let filename = match self.elements.get(index).unwrap() {
                    Element::Include(name) => name,
                    _ => unreachable!()
                };

                let contents = fs::read_to_string(filename)?;
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

    /// Returns a [`Document`] of the elements in the pattern
    ///
    /// Retreives the pattern `name`
    ///
    /// # Errors
    /// Returns a [`Error`] if the pattern does not exist.
    ///
    /// [`Error`]: enum.Error.html
    /// [`Document`]: struct.Document.html
    pub fn get_pattern(&self, name: &str) -> Result<Document, Error> {
        let compare = |element: &&Element| match element {
            Element::Pattern(pname, _) if pname == name => true,
            _ => false,
        };

        if let Some(pattern) = self.elements.iter().find(compare) {
            if let Element::Pattern(_, elements) = pattern {
                return Ok(Document {
                    elements: elements.to_vec(),
                });
            }
        }

        Err(Error::BadPattern(name.to_string()))
    }

    /// Returns a usize indicating how many elements were inserted into the
    /// document.
    ///
    /// Inserts the elementa from `pattern` into this document, before the
    /// pattern `name`
    ///
    /// # Errors
    /// Returns a [`Error`] if the pattern does
    /// not exist.
    ///
    /// [`Error`]: enum.Error.html
    pub fn set_pattern(&mut self, name: &str, pattern: Document) -> Result<usize, Error> {
        let compare = |element: &Element| match element {
            Element::Pattern(pname, _) if pname == name => true,
            _ => false,
        };

        if let Some(index) = self.elements.iter().position(compare) {
            let mut elements_inserted = 0;
            for element in pattern.elements.into_iter().rev() {
                self.elements.insert(index, element);
                elements_inserted += 1;
            }

            return Ok(elements_inserted);
        }

        Err(Error::BadPattern(name.to_string()))
    }

    /// Returns a string of this document.
    ///
    /// Only text is included in the string. No variables, unproccesed includes,
    /// or patterns will be in the string.
    pub fn as_string(self) -> String {
        let mut string = String::new();

        for element in self.elements {
            string.push_str(&element.string());
        }

        string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include() {
        let test_str = "Test{~ $var ~}{~ @include tests/testdoc ~}tesT";
        let cmp_before_vec = [
            Element::Text(String::from("Test")),
            Element::Variable(String::from("var")),
            Element::Include(String::from("tests/testdoc")),
            Element::Text(String::from("tesT")),
        ];
        let cmp_after_vec = [
            Element::Text(String::from("Test")),
            Element::Variable(String::from("var")),
            Element::Text(String::from("Testdoc!")),
            Element::Variable(String::from("var")),
            Element::Text(String::from("\n")),
            Element::Text(String::from("tesT")),
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_before_vec);

        let includes = doc.process_includes().unwrap();
        assert_eq!(1, includes);
        assert_eq!(doc.elements, cmp_after_vec);
    }

    #[test]
    fn test_nested_include() {
        let test_str = "{~ @include tests/testdoc_nested ~}";
        let cmp_before_vec = [Element::Include(String::from("tests/testdoc_nested"))];
        let cmp_after_vec = [
            Element::Text(String::from("Testdoc!")), // From testdoc
            Element::Variable(String::from("var")),  // From testdoc
            Element::Text(String::from("\n")),       // From testdoc
            Element::Text(String::from("\n")),       // From testdoc_nested
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
            Element::Variable(String::from("foo")),
        ];
        let cmp_after_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar")),
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
            Element::Variable(String::from("foo")),
        ];
        let cmp_after_foo_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar")),
            Element::Variable(String::from("foobar")),
            Element::Text(String::from("bar")),
        ];
        let cmp_after_foobar_vec = vec![
            Element::Text(String::from("Test!")),
            Element::Text(String::from("bar")),
            Element::Text(String::from("barfoo")),
            Element::Text(String::from("bar")),
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

    #[test]
    fn test_get_pattern() {
        let test_str = "Test{~ @pattern pat ~}{~ $var ~}{~ @end-pattern ~}";
        let cmp_pat_vec = vec![Element::Variable(String::from("var"))];
        let cmp_vec = vec![
            Element::Text(String::from("Test")),
            Element::Pattern(
                String::from("pat"),
                vec![Element::Variable(String::from("var"))],
            ),
        ];

        let doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_vec);

        let pattern = doc.get_pattern("pat").unwrap();
        assert_eq!(pattern.elements, cmp_pat_vec);
    }

    #[test]
    fn test_set_pattern() {
        let test_str = "Test!{~ @pattern pat ~}{~ $var ~}{~ @end-pattern ~}!tesT";
        let cmp_pat_vec = vec![Element::Variable(String::from("var"))];
        let cmp_vec_0 = vec![
            Element::Text(String::from("Test!")),
            Element::Pattern(
                String::from("pat"),
                vec![Element::Variable(String::from("var"))],
            ),
            Element::Text(String::from("!tesT")),
        ];
        let cmp_vec_1 = vec![
            Element::Text(String::from("Test!")),
            Element::Variable(String::from("var")),
            Element::Pattern(
                String::from("pat"),
                vec![Element::Variable(String::from("var"))],
            ),
            Element::Text(String::from("!tesT")),
        ];
        let cmp_vec_2 = vec![
            Element::Text(String::from("Test!")),
            Element::Variable(String::from("var")),
            Element::Text(String::from("rav")),
            Element::Pattern(
                String::from("pat"),
                vec![Element::Variable(String::from("var"))],
            ),
            Element::Text(String::from("!tesT")),
        ];

        let mut doc = Document::new(test_str);
        assert_eq!(doc.elements, cmp_vec_0);

        let pat = doc.get_pattern("pat").unwrap();
        assert_eq!(pat.elements, cmp_pat_vec);

        doc.set_pattern("pat", pat).unwrap();
        assert_eq!(doc.elements, cmp_vec_1);

        let mut pat = doc.get_pattern("pat").unwrap();
        assert_eq!(pat.elements, cmp_pat_vec);

        let variables = pat.set_variable("var", "rav");
        assert_eq!(1, variables);

        doc.set_pattern("pat", pat).unwrap();
        assert_eq!(doc.elements, cmp_vec_2);
    }

    #[test]
    fn test_to_string() {
        let test_str =
            "Hello, my name is {~ $name ~}!\nIncluding testdoc:\n\t{~ @include tests/testdoc ~}";
        let cmp_str =
            "Hello, my name is genbyte!\nIncluding testdoc:\n\tTestdoc! Variable replaced.\n";

        let mut doc = Document::new(test_str);
        doc.process_includes().unwrap();
        doc.set_variable("name", "genbyte");
        doc.set_variable("var", " Variable replaced.");

        assert_eq!(doc.as_string(), cmp_str);
    }
}
