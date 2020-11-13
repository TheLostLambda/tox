# Documentation

A library for evaluating math expressions.

## Published Versions

The trimmed `shunting` crate has has its latest version pushed to crates.io as `thin-shunting`

## Using the library

```rust
fn main() {
  let input = "sin(0.2)^2 + cos(0.2)^2";
  let expr = ShuntingParser::parse_str(input).unwrap();
  let result = MathContext::new().eval(&expr).unwrap();
  println!("{} = {}", expr, result);
}
```

## A MathContext

`MathContext` allows keeping context across multiple invocations to parse and evaluate. You can do this via the `setvar` method.

## Credit

The **vast** majority of the work here was done by Rodolfo Granata <warlock.cc@gmail.com>, I've just trimmed things down and cleaned up the code a little.