# Confie
Simple library that exposes ability to generate const configs from toml files.

## Usage
Here is the really simple config with 2 sections:

```toml 
# example.toml
[foo]
one = 1
string = "bar"

[bar]
negative_one = -1
array = ["one", "two", "three"]

```
So in Rust it is pretty straightforward. The library does not rely on serde. It takes the structures you provide and puts all values from each section into its constructor, using the Rust compiler to tell you if something is missing or has the wrong type.

**Supported types are**: Strings, Integers, Booleans, Floats and Arrays.
```rust

use confie::load;

struct FooConfig {
    one: u8,
    string: &'static str
}

struct BarConfig {
    negative_one: i8,
    array: &'static [&'static str]
}

confie::load!("./example.toml", {
    foo: FooConfig,
    bar: BarConfig,
});
 ```

The macro expands each provided struct into a `const` named by the capitalised section name.

```rust
pub const FOO: FooConfig = FooConfig {
    one: 1,
    string: "bar",
};

pub const BAR: BarConfig = BarConfig {
    negative_one: -1,
    array: &["one", "two", "three"],
};
```
