//! Collection of helpers and function handlers.
use serde::Serialize;
use std::collections::HashMap;

use crate::{
    escape::{html_escape, EscapeFn},
    helper::{
        BlockHelper,
        Helper,
        HelperRegistry,
        JsonHelper,
        WithHelper,
    },
    log::LogHelper,
    output::{Output, StringOutput},
    parser::ParserOptions,
    template::{Loader, Template, Templates},
    Error, Result,
};

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
            escape: Box::new(html_escape),
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

    /// Registry of helpers.
    pub fn helpers(&self) -> &HelperRegistry<'reg> {
        &self.helpers
    }

    /// Mutable reference to the helper registry.
    pub fn helpers_mut(&mut self) -> &mut HelperRegistry<'reg> {
        &mut self.helpers
    }

    /// Registry of templates.
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
    pub fn compile<'a>(
        &'a self,
        template: &'a str,
        options: ParserOptions,
    ) -> Result<'a, Template<'a>> {
        Templates::compile(template, options)
    }

    /// Render a template without registering it and return the result.
    pub fn once<'a, T>(
        &'a self,
        name: &str,
        template: &'a Template<'source>,
        data: &T,
    ) -> Result<'a, String>
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

    pub fn render<'a, T>(&'a self, name: &str, data: &T) -> Result<'a, String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        self.render_to_write(name, data, &mut writer)?;
        Ok(writer.into())
    }

    pub fn render_to_write<'a, T>(
        &'a self,
        name: &str,
        data: &T,
        writer: &mut impl Output,
    ) -> Result<'a, ()>
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
