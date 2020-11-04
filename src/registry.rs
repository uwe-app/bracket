//! Collection of helpers and function handlers.
use serde::Serialize;

use crate::{
    escape::{escape_html, EscapeFn},
    helper::HelperRegistry,
    output::{Output, StringOutput},
    parser::{Parser, ParserOptions},
    render::Render,
    template::{Template, Templates},
    Error, Result,
};

/// Registry is the entry point for compiling and rendering templates.
///
/// A template name is always required for error messages.
pub struct Registry<'reg, 'source> {
    helpers: HelperRegistry<'reg>,
    templates: Templates<'source>,
    escape: EscapeFn,
}

impl<'reg, 'source> Registry<'reg, 'source> {

    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            helpers: HelperRegistry::new(),
            templates: Default::default(),
            escape: Box::new(escape_html),
        }
    }

    /// Create a registry using a collection of templates.
    pub fn new_templates(templates: Templates<'source>) -> Self {
        let mut reg = Registry::new();
        reg.templates = templates;
        reg
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

    /// Template registry.
    ///
    /// For partials to be located they must exist in this
    /// templates collection.
    pub fn templates(&self) -> &Templates<'source> {
        &self.templates
    }

    /// Mutable reference to the templates registry.
    pub fn templates_mut(&mut self) -> &mut Templates<'source> {
        &mut self.templates
    }

    /// Compile a string to a template.
    pub fn compile(
        &self,
        template: &'source str,
        options: ParserOptions,
    ) -> Result<Template<'source>> {
        Templates::compile(template, options)
    }

    /// Render a template without registering it and return 
    /// the result as a string.
    ///
    /// This function buffers the template nodes before rendering; if low 
    /// latency is required use the stream functions.
    pub fn once<T>(
        &self,
        name: &str,
        source: &str,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        let template = self.compile(source, ParserOptions::new(name.to_string()))?;
        template.render(
            self.escape(),
            self.helpers(),
            self.templates(),
            name,
            data,
            &mut writer,
        )?;
        Ok(writer.into())
    }

    /// Stream a dynamic template and buffer the result to a string.
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
        let mut rc = Render::new(
            self.escape(),
            self.helpers(),
            self.templates(),
            source,
            data,
            Box::new(writer),
        )?;

        let mut parser = Parser::new(source, options);
        for node in parser {
            let node = node?;
            // FIXME: implement this, currently not working as we store the 
            // FIXME: next and previous nodes in the renderer which means 
            // FIXME: node is not living long enough for the renderer to 
            // FIXME: do it's job.
            //rc.render_node(&node)?;
        }
        Ok(())
    }

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

    /// Render a compiled template without registering it and return 
    /// the result as a string.
    pub fn render_template<T>(
        &self,
        name: &str,
        template: &Template<'source>,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        template.render(
            self.escape(),
            self.helpers(),
            self.templates(),
            name,
            data,
            &mut writer,
        )?;
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
        tpl.render(
            self.escape(),
            self.helpers(),
            self.templates(),
            name,
            data,
            writer,
        )?;
        Ok(())
    }
}
