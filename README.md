# protobuf-to-json

[![Crates.io](https://img.shields.io/crates/v/protobuf-to-json.svg)](https://crates.io/crates/protobuf-to-json)
[![rustdoc](https://img.shields.io/badge/Doc-protobuf_to_json-green.svg)](https://docs.rs/protobuf-to-json/)

A converter that parses arbitrary protobuf data to json

## Features
* No schema required
* Use field number as json key
* Configurable bytes encoding (base64, hex, byte array, etc.)
* Guess length-delimited value types (string, nested message, bytes)

## Limitations
* Length-delimited value type is guessed based on content. It may not always be correct.
* Repeated fields (with the same field number) may not be grouped into arrays when only one field is parsed.

## Installation
Add it to your `Cargo.toml`:

```toml
[dependencies]
protobuf-to-json = "0.1"
```

## Example

``` rust
use protobuf_to_json::Parser;
use serde_json::json;

let data = [
        0x0d, 0x1c, 0x00, 0x00, 0x00, 0x12, 0x03, 0x59, 0x6f, 0x75, 0x1a, 0x02, 0x4d, 0x65,
        0x20, 0x2b, 0x2a, 0x0a, 0x0a, 0x06, 0x61, 0x62, 0x63, 0x31, 0x32, 0x33, 0x12, 0x00,
    ];
let parser = Parser::new();
let json = parser.parse(&data).unwrap();
println!("{}", json);
let expected = json!({
    "1": 28,
    "2": "You",
    "3": "Me",
    "4": 43,
    "5": {
        "1": "abc123",
        "2": ""
    }
});
assert_eq!(json, expected);
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.