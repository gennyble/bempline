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

   /*pub fn set_variable(&mut self, key: &str, value: &str) -> usize {
        /*
        Replace all instances of Element::Variable(key) with
            Element::Text(value).
        Make sure to NOT replace anything in a template
        return number of times it was replaced
        */
    }

    /*pub fn process_includes(&mut self) -> Result<Error, usize>{
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

    pub fn get_template(&self, name: &str) -> Result<Error, Document> {
        /*
        Find the TemplateStart with the name `name`
        Work until the first TemplateEnd, tempaltes can't overlap
        Return these elements as a new document
        */
    }

    pub fn as_string() -> String {
        /*
        Maybe this would be better as like IntoString
        make sure to remove anything that isn't Text
        also remove everything in between and including templates
        cat all the Element::Text-s together
        */
    }*/
}
