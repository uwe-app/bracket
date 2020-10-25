use std::collections::HashMap;

use serde::Serialize;

use crate::{
    error::{RenderError, SyntaxError},
    output::{Output, StringOutput},
    render::{RenderContext, RenderState, Renderer},
    lexer::parser::ParserOptions,
    template::Template,
    Error, Result,
};

pub struct Registry<'reg> {
    templates: HashMap<&'reg str, Template<'reg>>,
}

impl<'reg> Registry<'reg> {
    pub fn new() -> Self {
        Self {
            templates: Default::default(),
        }
    }

    pub fn compile<'source>(s: &'source str, options: ParserOptions) -> std::result::Result<Template, SyntaxError> {
        Template::compile(s, options)
    }

    pub fn templates(&self) -> &HashMap<&str, Template<'reg>> {
        &self.templates
    }

    pub fn register_template(
        &mut self,
        name: &'reg str,
        template: Template<'reg>,
    ) {
        self.templates.insert(name, template);
    }

    pub fn unregister_template(
        &mut self,
        name: &'reg str,
    ) -> Option<Template<'reg>> {
        self.templates.remove(name)
    }

    pub fn get_template(&self, name: &'reg str) -> Result<&Template<'reg>> {
        self.templates.get(name).ok_or_else(|| {
            Error::from(RenderError::TemplateNotFound(name.to_string()))
        })
    }

    pub fn register_template_string(
        &mut self,
        name: &'reg str,
        s: &'reg str,
        options: ParserOptions,
    ) -> Result<()> {
        let tpl = Registry::compile(s, options)?;
        Ok(self.register_template(name, tpl))
    }

    pub fn render<T>(&self, name: &'reg str, data: &T) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        self.render_to_write(name, data, &mut writer)?;
        Ok(writer.into())
    }

    pub fn render_to_write<T>(
        &self,
        name: &'reg str,
        data: &T,
        writer: &mut impl Output,
    ) -> Result<()>
    where
        T: Serialize,
    {
        let tpl = self.get_template(name)?;
        let state = RenderState::new();
        let mut rc = RenderContext::new(&self, data, state, Box::new(writer))?;
        //println!("Do a render {:?}", tpl);
        tpl.render(&mut rc)?;

        Ok(())
    }
}
