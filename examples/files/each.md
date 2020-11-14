# {{title}}

The `each` helper can be used to iterate arrays or objects; when iterating an array we can access the `@index` local variable:

```text
{{~#each list}}
Item: {{this}}, Index: {{@index}}, First: {{@first}}, Last: {{@last}}
{{~/each}}
```

When we iterate objects we can also access the `@key` field:

```text
{{~#each map}}
{{@key}} = {{this}}, Index: {{@index}}, First: {{@first}}, Last: {{@last}}
{{~/each}}
```
