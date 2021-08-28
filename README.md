# bempline
A simple template engine for simple things.

### Syntax
Variables are alphanumeric strings (underscores, too) surrounded by braces. Here's an `{example}`.

You can prevent `{word}` from being seen as a variable by escaping the opening brace. Like `\{this}`.

## Example
If you have this document in something like `template.bpl`
```
Dear {name},

Some generic email text here!

Sincerely,
Some Company
```

You can fill it out for the named `Ferris` and `Rusty` like so
```rust
use bempline::Document;

fn main() {
	let doc = Document::from_file("template.bpl").unwrap();
	let names = vec!["Ferris", "Rusty"];

	for name in names {
		let mut cloned = doc.clone();
		cloned.set("name", name);
		
		println!("{}", cloned.compile());
	}
}
```