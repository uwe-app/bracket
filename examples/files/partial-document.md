# {{title}}

## Partial

A named partial which receives some data via hash parameters.

---
{{> partial-named message="Hello partial!"~}}
---

## Dynamic Partial

A partial evaluated from the variable `partial-name` using a sub-expression.

---

{{> (partial-name) message="Hello dynamic partial!"}}

---

## Partial Block

A partial block which renders a template passed using the special `@partial-block` variable.

---

{{# > partial-block message="Hello partial block!"}}
This is some content for the `@partial-block` variable.
{{/partial-block}}

---
