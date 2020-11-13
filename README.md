# Bracket

Alternative template engine using handlebars-compatible syntax designed for speed, correctness and excellent error handling.

Inspired by [handlebars-rust][] but the API is incompatible in certain places where we think the design should be different. 

Except for partials the library does not take ownership of the underlying template strings it simply references them as string slices so it is the caller's responsibility to store them; as a convenience a `Loader` can be used to store and load templates from disc.

## Features

The default features are batteries included but you can set `default-features = false` and cherry pick.

* `helpers`: Include all helpers.
* `log-helper`: Enable the `log` helper.
* `each-helper`: Enable the `each` helper.
* `with-helper`: Enable the `with` helper.
* `lookup-helper`: Enable the `lookup` helper.
* `json-helper`: Enable the `json` helper.
* `conditional-helper`: Enable the `if` and `unless` helpers.
* `logical-helper`: Enable the `and`, `or` and `not` helpers.
* `comparison-helper`: Enable the `eq`, `ne`, `gt`, `lt`, `gte` and `lte` helpers.
* `stream`: Enable the `stream` functions on the registry.
* `fs`: Support loading templates from the filesystem.

## Lifetimes

* `'reg` The lifetime of the registry; helpers, partials and escape functions.
* `'source` The lifetime of a source template string.
* `'render` The lifetime of a template render.
* `'call` The lifetime of a helper call.

[handlebars-rust]: https://github.com/sunng87/handlebars-rust/

