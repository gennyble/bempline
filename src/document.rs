use crate::elements::Element;

struct Document {
    elements: Vec<Element>
}

impl Document {
    pub fn new(text: &str) -> Self {
        Document {
            elements: Element::parse_elements(text)
        }
    }

    pub fn process_includes(&mut self) {
        /*
        @START
        Use binary_search to find an include, any include.
        Get the include from the vector
        Get the file the include points to
        Element::parse_elements on the contents of the file
        Insert contents directly before the include
        remove the include
        @goto start while no more includes
        */
    }
}
