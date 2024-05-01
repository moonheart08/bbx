# BBX
![Crates.io MSRV](https://img.shields.io/crates/msrv/bbx)
![Crates.io License](https://img.shields.io/crates/l/bbx)
<a href="https://crates.io/crates/bbx">
![Crates.io Version](https://img.shields.io/crates/v/bbx)
</a>
<a href="https://docs.rs/bbx/latest/bbx/">
![docs.rs](https://img.shields.io/docsrs/bbx)
</a>
<a href="https://discord.gg/ChVzW85C">
![Discord](https://img.shields.io/discord/1170413428393902170?link=https%3A%2F%2Fdiscord.gg%2FChVzW85C)
</a>

A robust and performant (constant time, no recursion) BBCode pull parser with `no_std`/`alloc` support.

# Examples
## Quick parsing
```rs
// Parse a document, throwing all of its component tokens into the console.
let mut parser = BBParser::new(input);

for token in parser {
    println!("{:?}", token);
}
```

## Quick sanitized HTML output
```rs
// Simple serializer default with all of the v1.0.0 (or earlier) tags considered "core" to the library.
let mut serializer: HtmlSerializer<SimpleHtmlWriter> = 
    HtmlSerializer::with_tags(all_core_v1_tags());
let mut parser = BBParser::new(input);
println!("Document:");
println!("{}", serializer.serialize(parser));
```