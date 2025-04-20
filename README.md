# `wikitext_simplified`

Wraps around `parse-wiki-text-2` to create a simplified and nested AST for parsed wikitext.

`wikitext_util` contains helper functions for working with `parse-wiki-text-2`'s AST.

Split out from [`genresin.space`'s repository](https://github.com/genresinspace/genresinspace.github.io/tree/228fd864b80e1517d4dbee0588dc98ddcbf31a35/wikitext_simplified).

## Features

- Simplified AST structure for wikitext
- WASM support for web-based applications
- Helper utilities for text extraction and node manipulation
- Wikipedia-compatible configuration

## Example

```rust
use wikitext_simplified::parse_and_simplify_wikitext;

let wikitext = "This is '''bold''' and ''italic'' text with a [[link]].";
let nodes = parse_and_simplify_wikitext(wikitext);
// nodes will contain a simplified AST of the wikitext
```
