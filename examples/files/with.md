# {{title}}

Use the `with` helper to change the current scope.

This example sets the `list` variable as the current scope and prints it using the `json` helper:

```json
{{~#with list}}
{{json this pretty=true}}
{{/with~}}
```
