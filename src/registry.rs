//! Primary entry point for compiling and rendering templates.
use serde::Serialize;
use std::convert::TryFrom;

use crate::{
    escape::{self, EscapeFn},
    helper::HelperRegistry,
    output::{Output, StringOutput},
    parser::{Parser, ParserOptions},
    template::{Template, Templates, Loader},
    Error, Result,
};

/// Registry is the entry point for compiling and rendering templates.
///
/// A template name is always required for error messages.
pub struct Registry<'reg, 'source> {
    loader: Loader,
    helpers: HelperRegistry<'reg>,
    templates: Templates<'source>,
    escape: EscapeFn,
    strict: bool,
}

impl<'reg, 'source> Registry<'reg, 'source> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            loader: Default::default(),
            helpers: HelperRegistry::new(),
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

    /// Set the registry templates collection.
    pub fn set_templates(&mut self, templates: Templates<'source>) {
        self.templates = templates;
    }

    /// Set a loader for this registry.
    ///
    /// All sources in the loader are compiled as templates and assigned 
    /// to the templates collection of this registry.
    pub fn set_loader(&'source mut self, loader: Loader) -> Result<()> {
        self.loader = loader;
        let templates = Templates::try_from(&self.loader)?;
        self.templates = templates;
        Ok(())
    }

    /// Compile a string to a template.
    pub fn compile(
        &self,
        template: &'source str,
        options: ParserOptions,
    ) -> Result<Template<'source>> {
        Templates::compile(template, options)
    }

    /// Compile a string to a template using the given name.
    pub fn parse(
        &self,
        name: &str,
        template: &'source str,
    ) -> Result<Template<'source>> {
        self.compile(template, ParserOptions::new(name.to_string(), 0, 0))
    }

    /// Lint a template.
    pub fn lint(
        &self,
        name: &str,
        template: &'source str,
    ) -> Result<Vec<Error>> {
        let mut errors: Vec<Error> = Vec::new();
        let mut parser =
            Parser::new(template, ParserOptions::new(name.to_string(), 0, 0));
        parser.set_errors(&mut errors);
        for _ in parser {}
        Ok(errors)
    }

    /// Render a template without registering it and return
    /// the result as a string.
    ///
    /// This function buffers the template nodes before rendering.
    pub fn once<T>(&self, name: &str, source: &str, data: &T) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        let template =
            self.compile(source, ParserOptions::new(name.to_string(), 0, 0))?;
        template.render(
            self.strict(),
            self.escape(),
            self.helpers(),
            self.templates(),
            name,
            data,
            &mut writer,
        )?;
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
    pub fn render_template<T>(
        &self,
        name: &str,
        template: &Template<'source>,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        //let mut local_helpers = HelperRegistry::new();
        let mut writer = StringOutput::new();
        template.render(
            self.strict(),
            self.escape(),
            self.helpers(),
            //&mut local_helpers,
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
            self.strict(),
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

/// Create a registry using a collection of templates.
impl<'reg, 'source> From<Templates<'source>> for Registry<'reg, 'source> {
    fn from(templates: Templates<'source>) -> Self {
        let mut reg = Registry::new();
        reg.templates = templates;
        reg
    }
}

/// Create a registry using a collection of helpers.
impl<'reg, 'source> From<HelperRegistry<'reg>> for Registry<'reg, 'source> {
    fn from(helpers: HelperRegistry<'reg>) -> Self {
        let mut reg = Registry::new();
        reg.helpers = helpers;
        reg
    }
}

/*
impl<'reg, 'source> TryFrom<(&'source Loader, HelperRegistry<'reg>)> for Registry<'reg, 'source> {
    type Error = crate::error::Error;
    fn try_from(
        value: (&'source Loader, HelperRegistry<'reg>),
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Registry{
            templates: Templates::try_from(value.0)?,
            helpers: value.1,
            escape: Box::new(escape::html),
            strict: false,
        })
    }
}
*/
