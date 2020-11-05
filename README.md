# Bracket

Alternative template engine using handlebars-compatible syntax designed for speed, correctness and excellent error handling.

Inspired by [handlebars-rust][] but the API is incompatible in certain places where we think the design should be different. In particular this library differentiates between normal helpers and block helpers and is designed to cater better to templates that are not tied to the registry lifetime (dynamic templates).

Except for partials the library does not take ownership of the underlying template strings it simply references them as string slices so it is the caller's responsibility to store them; as a convenience a `Loader` can be used to store and load templates from disc.

## Features

The default features are batteries included but you can set `default-features = false` and cherry pick.

* `helpers`: Include all helpers.
* `log-helper`: Enable the `log` helper.
* `lookup-helper`: Enable the `lookup` helper.
* `json-helper`: Enable the `json` helper.
* `logical-helper`: Enable the `and`, `or` and `not` helpers.

## Lifetimes

* `'reg` The lifetime of the registry; helpers, partials and escape functions.
* `'source` The lifetime of a source template string.
* `'render` The lifetime of a template render.

[handlebars-rust]: https://github.com/sunng87/handlebars-rust/

