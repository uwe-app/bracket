# {{title}}

To debug our templates we can use the `log` helper: `\{{log "Message to print"}}`.

{{~log "Trace message" level="trace"~}}
{{log "Debug message" level="debug"~}}
{{log "Warn message" level="warn"~}}
{{log "Info message"~}}
{{log "Error message" level="error"}}

Use sub expressions to combine logging with JSON evaluation `\{{log (json this)}}`.

{{log (json this pretty=true)~}}
