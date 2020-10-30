# {{title}}

This is a markdown document using handlebars for templating. The first header is a basic variable substitution but we can also use explicit `@root` references too, here is a block quote that shows the page title:

> {{@root.title}}

To help debug our templates we can use the `log` helper: `\{{log "Message to print"}}`.

{{~log "Trace message" level="trace"~}}
{{log "Debug message" level="debug"~}}
{{log "Warn message" level="warn"~}}
{{log "Info message"~}}
{{log "Error message" level="error"}}

```
{{{this}}}
```

{{#with list}}
```
{{{this}}}
```
{{/with}}

\{{expr}}
{{{unescaped}}}

{{var foo="bar"}}

{{> (var)}}

\{{ > }}
\{{ > a.b}}

{{{{  raw }}}}
This is some raw text {{inline-raw}}.
{{{{/raw}}}}

{{# test-block}}
This is some block text with an {{inline}}
{{/test-block}}

{{> partial}}

{{#> partial-block}}
{{@partial-block}}
{{/partial-block}}

{{!-- a comment --}}

{{foo null true false -20 3.14 2E+2 "blah\"baz"}}

\{{[1,2,3]}}
\{{false}}
\{{null}}

\{{foo {"a": "b"}}}
