//! Primary entry point for compiling and rendering templates.
use serde::Serialize;

#[cfg(feature = "fs")]
use std::ffi::OsStr;
#[cfg(feature = "fs")]
use std::path::Path;

use crate::{
    escape::{self, EscapeFn},
    helper::{HandlerRegistry, HelperRegistry},
    output::{Output, StringOutput},
    parser::{Parser, ParserOptions},
    render::CallSite,
    template::{Template, Templates},
    Error, Result,
};

/// Registry is the entry point for compiling and rendering templates.
///
/// A template name is always required for error messages.
pub struct Registry<'reg> {
    helpers: HelperRegistry<'reg>,
    handlers: HandlerRegistry<'reg>,
    templates: Templates,
    escape: EscapeFn,
    strict: bool,
}

impl<'reg> Registry<'reg> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            helpers: HelperRegistry::new(),
            handlers: Default::default(),
            templates: Default::default(),
            escape: Box::new(escape::html),
            strict: false,
        }
    }

    /// Set the strict mode.
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict
    }

    /// Get the strict mode.
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Set the escape function for rendering.
    pub fn set_escape(&mut self, escape: EscapeFn) {
        self.escape = escape;
    }

    /// The escape function to use for rendering.
    pub fn escape(&self) -> &EscapeFn {
        &self.escape
    }

    /// Helper registry.
    pub fn helpers(&self) -> &HelperRegistry<'reg> {
        &self.helpers
    }

    /// Mutable reference to the helper registry.
    pub fn helpers_mut(&mut self) -> &mut HelperRegistry<'reg> {
        &mut self.helpers
    }

    /// Event handler registry.
    pub fn handlers(&self) -> &HandlerRegistry<'reg> {
        &self.handlers
    }

    /// Mutable reference to the event handler registry.
    pub fn handlers_mut(&mut self) -> &mut HandlerRegistry<'reg> {
        &mut self.handlers
    }

    /// Templates collection.
    pub fn templates(&self) -> &Templates {
        &self.templates
    }

    /// Get a named template.
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Insert a named string template.
    pub fn insert<N, C>(&mut self, name: N, content: C) -> Result<()>
    where
        N: AsRef<str>,
        C: AsRef<str>,
    {
        let name = name.as_ref().to_owned();
        let template = self.compile(
            content.as_ref().to_owned(),
            ParserOptions::new(name.clone(), 0, 0),
        )?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Add a named template from a file.
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn add<P>(&mut self, name: String, file: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file_name = file
            .as_ref()
            .to_string_lossy()
            .into_owned()
            .to_string();

        let (_, content) = self.read(file)?;
        let template =
            self.compile(content, ParserOptions::new(file_name, 0, 0))?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Load a file and use the file path as the template name.
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn load<P: AsRef<Path>>(&mut self, file: P) -> Result<()> {
        let file_name = file
            .as_ref()
            .to_string_lossy()
            .into_owned()
            .to_string();

        let (name, content) = self.read(file)?;
        let template =
            self.compile(content, ParserOptions::new(file_name, 0, 0))?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Load all the files in a target directory that match the
    /// given extension.
    ///
    /// The generated name is the file stem; ie, the name of the file
    /// once the extension has been removed.
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn read_dir<P: AsRef<Path>>(
        &mut self,
        file: P,
        extension: &str,
    ) -> Result<()> {
        let ext = OsStr::new(extension);
        for entry in std::fs::read_dir(file.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == ext {
                        let file_name = path
                            .to_string_lossy()
                            .into_owned()
                            .to_string();

                        let name = path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_owned()
                            .to_string();
                        let (_, content) = self.read(path)?;
                        let template = self.compile(
                            content,
                            ParserOptions::new(file_name, 0, 0),
                        )?;
                        self.templates.insert(name, template);
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "fs")]
    fn read<P: AsRef<Path>>(
        &self,
        file: P,
    ) -> std::io::Result<(String, String)> {
        let path = file.as_ref();
        let name = path.to_string_lossy().to_owned().to_string();
        let content = std::fs::read_to_string(path)?;
        Ok((name, content))
    }

    /// Compile a string to a template.
    ///
    /// To compile a template and add it to this registry use [insert()](Registry#method.insert),
    /// [add()](Registry#method.add), [load()](Registry#method.load) or [read_dir()](Registry#method.read_dir).
    pub fn compile<'a, S>(
        &self,
        template: S,
        options: ParserOptions,
    ) -> Result<Template>
    where
        S: AsRef<str>,
    {
        Ok(Template::compile(template.as_ref().to_owned(), options)?)
    }

    /// Compile a string to a template using the given name.
    ///
    /// This is a convenience function for calling [compile()](Registry#method.compile)
    /// using parser options with the given name.
    pub fn parse<'a, S>(&self, name: &str, template: S) -> Result<Template>
    where
        S: AsRef<str>,
    {
        self.compile(template, ParserOptions::new(name.to_string(), 0, 0))
    }

    /// Lint a template.
    pub fn lint<S>(&self, name: &str, template: S) -> Result<Vec<Error>>
    where
        S: AsRef<str>,
    {
        let mut errors: Vec<Error> = Vec::new();
        let mut parser = Parser::new(
            template.as_ref(),
            ParserOptions::new(name.to_string(), 0, 0),
        );
        parser.set_errors(&mut errors);
        for _ in parser {}
        Ok(errors)
    }

    /// Render a template without registering it and return
    /// the result as a string.
    ///
    /// This function buffers the template nodes before rendering.
    pub fn once<T, S>(&self, name: &str, source: S, data: &T) -> Result<String>
    where
        T: Serialize,
        S: AsRef<str>,
    {
        let mut writer = StringOutput::new();
        let template = self.compile(
            source.as_ref(),
            ParserOptions::new(name.to_string(), 0, 0),
        )?;
        template.render(self, name, data, &mut writer, Default::default())?;
        Ok(writer.into())
    }

    /// Render a template without registering it and return
    /// the result as a string using an existing call stack.
    ///
    /// This function buffers the template nodes before rendering.
    ///
    /// Use this function if you need to render a string inside a
    /// helper definition but want to respect the call stack
    /// of the existing render, for example:
    ///
    /// ```ignore
    /// let result = rc
    ///     .registry()
    ///     .once_stack(template_path, &content, rc.data(), rc.stack().clone())
    ///     .map_err(|e| {
    ///         HelperError::new(e.to_string())
    ///     })?;
    /// rc.write(&result)?;
    /// ```
    pub(crate) fn once_stack<T, S>(
        &self,
        name: &str,
        source: S,
        data: &T,
        stack: Vec<CallSite>,
    ) -> Result<String>
    where
        T: Serialize,
        S: AsRef<str>,
    {
        let mut writer = StringOutput::new();
        let template = self.compile(
            source.as_ref(),
            ParserOptions::new(name.to_string(), 0, 0),
        )?;
        template.render(self, name, data, &mut writer, stack)?;
        Ok(writer.into())
    }

    /*

    /// Stream a dynamic template and buffer the result to a string.
    ///
    /// Requires the `stream` feature.
    #[cfg(feature = "stream")]
    pub fn stream<T>(
        &self,
        name: &str,
        source: &str,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        let options = ParserOptions::new(name.to_string());
        self.stream_to_write(name, source, data, &mut writer, options)?;
        Ok(writer.into())
    }

    /// Stream a dynamic template to a writer.
    ///
    /// Requires the `stream` feature.
    #[cfg(feature = "stream")]
    pub fn stream_to_write<T>(
        &self,
        name: &str,
        source: &str,
        data: &T,
        writer: &mut impl Output,
        options: ParserOptions,
    ) -> Result<()>
    where
        T: Serialize,
    {
        let mut buffer: Vec<Node<'_>> = Vec::new();
        let mut rc = Render::new(
            self.strict(),
            self.escape(),
            self.helpers(),
            self.templates(),
            source,
            data,
            Box::new(writer),
        )?;

        // FIXME: implement this, currently not working as we store the
        // FIXME: next and previous nodes in the renderer which means
        // FIXME: node is not living long enough for the renderer to
        // FIXME: do it's job.
        let parser = Parser::new(source, options);
        let hint: Option<TrimHint> = Default::default();
        for node in parser {
            let node = node?;
            //let node = buffer.last().unwrap();
            for event in node.iter().trim(hint) {
                println!("{:#?}", event.node);
                //rc.render_node(event.node, event.trim)?;
            }
            //buffer.push(node);
        }

        drop(buffer);

        Ok(())
    }
    */

    /// Render a named template and buffer the result to a string.
    ///
    /// The named template must exist in the templates collection.
    pub fn render<T>(&self, name: &str, data: &T) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        self.render_to_write(name, data, &mut writer)?;
        Ok(writer.into())
    }

    /// Render a compiled template without registering it and
    /// buffer the result to a string.
    pub fn render_template<'a, T>(
        &self,
        name: &str,
        template: &Template,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        template.render(self, name, data, &mut writer, Default::default())?;
        Ok(writer.into())
    }

    /// Render a named template to a writer.
    ///
    /// The named template must exist in the templates collection.
    pub fn render_to_write<T>(
        &self,
        name: &str,
        data: &T,
        writer: &mut impl Output,
    ) -> Result<()>
    where
        T: Serialize,
    {
        let tpl = self
            .templates
            .get(name)
            .ok_or_else(|| Error::TemplateNotFound(name.to_string()))?;
        tpl.render(self, name, data, writer, Default::default())?;

        Ok(())
    }
}
