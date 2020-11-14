# {{title}}

Use the `if` helper to render conditionally.

```markdown
{{~#if true}}
This is a conditional `if` value.
{{/if~}}
```

```markdown
{{~#if false}}
ERROR!
{{else if true}}
This is a conditional `else if` value.
{{/if~}}
```

```markdown
{{~#if false}}
ERROR!
{{else if false}}
ERROR!
{{else}}
This is a conditional `else` value.
{{/if~}}
```
