# {{title}}

This is a markdown document using handlebars for templating. The first header is a basic variable substitution but we can also use explicit `@root` references too, here is a block quote that shows the page title:

> {{@root.title}}

To help debug our templates we can use the `log` helper: `\{{log "Message to print"}}`.

{{~log "Trace message" level="trace"~}}
{{log "Debug message" level="debug"~}}
{{log "Warn message" level="warn"~}}
{{log "Info message"~}}
{{log "Error message" level="error"}}

The `json` helper is useful for debugging template data, for example: `\{{json this}}` yields:

```
{{json this}}
```

## With

Use the `with` helper to change the current scope:

{{#with list}}
```
{{{this}}}
```
{{/with}}

{{> partial}}