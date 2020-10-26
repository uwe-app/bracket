use std::collections::HashMap;

use serde::Serialize;

use crate::{
    error::RenderError,
    helper::{
        BlockHelper, EachHelper, Helper, IfHelper, LogHelper, LookupHelper,
        UnlessHelper, WithHelper,
    },
    lexer::parser::ParserOptions,
    output::{Output, StringOutput},
    template::Template,
    Error, Result,
};

pub struct Registry<'reg> {
    templates: HashMap<&'reg str, Template<'reg>>,
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
}

impl<'reg, 'source> Registry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            templates: Default::default(),
            helpers: Default::default(),
            block_helpers: Default::default(),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        self.register_helper("log", Box::new(LogHelper {}));
        self.register_helper("lookup", Box::new(LookupHelper {}));

        self.register_block_helper("with", Box::new(WithHelper {}));
        self.register_block_helper("each", Box::new(EachHelper {}));
        self.register_block_helper("if", Box::new(IfHelper {}));
        self.register_block_helper("unless", Box::new(UnlessHelper {}));
    }

    pub fn register_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn Helper + 'reg>,
    ) {
        self.helpers.insert(name, helper);
    }

    pub fn register_block_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn BlockHelper + 'reg>,
    ) {
        self.block_helpers.insert(name, helper);
    }

    pub fn helpers(&self) -> &HashMap<&'reg str, Box<dyn Helper + 'reg>> {
        &self.helpers
    }

    pub fn block_helpers(
        &self,
    ) -> &HashMap<&'reg str, Box<dyn BlockHelper + 'reg>> {
        &self.block_helpers
    }

    pub fn compile(
        s: &'source str,
        options: ParserOptions,
    ) -> Result<Template> {
        Ok(Template::compile(s, options).map_err(Error::from)?)
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
        source: &'reg str,
        options: ParserOptions,
    ) -> Result<()> {
        let tpl = Registry::compile(source, options)?;
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
        tpl.render(self, name, data, writer)?;
        Ok(())
    }
}
