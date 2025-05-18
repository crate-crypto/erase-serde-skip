# erase-serde-skip

> **A tiny proc-macro that erases every `skip_serializing_if = ...` from field-level `#[serde(...)]` attributes, while leaving all other `serde` options—and every non-serde attribute—untouched.**

## WARNING

Do not use in production, has not been extensively tested and is likely very broken.

##  Why?

When you **serialize → deserialize** with Serde, any field marked

```rust
#[serde(skip_serializing_if = "...")]
```

is omitted whenever the predicate is true (eg `Option::is_none`).

This breaks a lot of binary serialization crates like `bincode`.

## How?

You can add an attribute on top of your struct and it will erase the `skip_serializing_if` predicates. You likely
want to put the attribute behind a feature flag for when you are serializing using something like `bincode`.

### Example 1

```rust
use erase_serde_skip::erase_skip_serializing_if;

#[erase_skip_serializing_if]
#[derive(serde::Serialize)]
struct Foo {
    a: u8,
    #[serde(skip_serializing_if = "Option::is_none", rename = "id")]
    b: Option<u32>,
}
```

After macro expansion this should be:

```rust
#[derive(serde::Serialize)]
struct Foo {
    a: u8,
    #[serde(rename = "id")]   // <- skip_serializing_if removed
    b: Option<u32>,
}
```

### Example 2

```rust
use erase_serde_skip::erase_skip_serializing_if;
use serde::{Serialize, Deserialize};
use bincode;

#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Example {
    #[serde(skip_serializing_if = "Option::is_none")]
    maybe: Option<u32>,
}

fn main() {
    let original = Example { maybe: None };

    let bytes = bincode::serialize(&original).unwrap();
    let decoded: Example = bincode::deserialize(&bytes).unwrap();

    assert_eq!(original, decoded);
}
```

## Limitations

- If you've renamed the serde dependency then this crate will leave that field untouched

## MSRV

This crate uses syn 2.0, which depends on Rust 1.70+.

##  License

Licensed under either of

- Apache License, Version 2.0
- MIT License
