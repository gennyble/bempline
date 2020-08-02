extern crate bempline;

use bempline::Document;

#[test]
fn test() {
    let test_str = "My name is {~ $name ~} and I'm from {~ $location ~}!";
    let cmp_str = "My name is Bempline and I'm from SourceHut!";

    let mut doc = Document::new(test_str);
    doc.set_variable("name", "Bempline");
    doc.set_variable("location", "SourceHut");

    assert_eq!(doc.as_string().unwrap(), cmp_str);
}
