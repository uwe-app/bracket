# {{title}}

This is a markdown document using handlebars for templating. The first header is a basic variable substitution but we can also use explicit `@root` references too, here is a block quote that shows the page title with escaping disabled:

> {{{@root.title}}}

The number is: {{list.[1]}}
The lookup value is: {{lookup foo.bar "qux"}}

## Log

To help debug our templates we can use the `log` helper: `\{{log "Message to print"}}`.

{{~log "Trace message" level="trace"~}}
{{log "Debug message" level="debug"~}}
{{log "Warn message" level="warn"~}}
{{log "Info message"~}}
{{log "Error message" level="error"}}

## JSON

The `json` helper is useful for debugging template data, for example: `\{{json this}}` yields:

```json
{{json this}}
```

If you want compact output pass a *truthy* value for the second argument: `\{{json this true}}`.

## Debugging

Use sub expressions to combine logging with JSON evaluation `\{{log (json this)}}`.

\{{log (json this)}}

## With

Use the `with` helper to change the current scope, here we select the `list` variable:

{{#with list}}
```json
{{this}}
```
{{/with}}

## Each

The `each` helper can be used to iterate arrays or objects. First let's iterate a list of numbers:

{{#each list}}
* Item: {{this}}, Index: {{@index}}, First: {{@first}}, Last: {{@last}}
{{/each}}

When we iterate objects we can also access the `@key` field:

{{#each map}}
* {{@key}} = {{this}}, Index: {{@index}}, First: {{@first}}, Last: {{@last}}
{{/each}}

{{> partial message="Hello from document"}}
