# {{title}}

\{{}}

{{foobar}}

This is a markdown document using handlebars for templating. The first header is a basic variable substitution but we can also use explicit `@root` references too, here is a block quote that shows the page title with escaping disabled:

> {{{@root.title}}}

{{# if title }}
Got a title :: {{title}}
{{ else if true }}
Got chained conditional 
{{ else }}
No title available
{{ /if}}

The number is: {{list.[1]}}
The lookup value is: {{lookup foo.bar "qux"}}

## JSON

The `json` helper is useful for debugging template data, for example: `\{{json this}}` yields:

```json
{{{json this pretty=true}}}
```

If you want pretty output pass a *truthy* value for the `pretty` hash parameter: `\{{json this pretty=true}}`.
