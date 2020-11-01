//! Main entry point for compiling, storing and rendering templates.
use std::collections::HashMap;
use serde::Serialize;

use crate::{
    escape::{html_escape, EscapeFn},
    helper::{
        BlockHelper,
        Helper,
        //EachHelper, Helper, IfHelper, LookupHelper, UnlessHelper,
        //WithHelper,
        JsonHelper,
        WithHelper,
    },
    log::LogHelper,
    output::{Output, StringOutput},
    parser::ParserOptions,
    template::{Template, Templates},
    Error, Result,
};

pub struct Registry<'reg> {
    //templates: HashMap<&'source str, Template<'source>>,
    //templates: Templates<'source>,
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
    escape: EscapeFn,
}

impl<'reg> Registry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            helpers: Default::default(),
            block_helpers: Default::default(),
            escape: Box::new(html_escape),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        self.register_helper("log", Box::new(LogHelper {}));
        self.register_helper("json", Box::new(JsonHelper {}));
        //self.register_helper("lookup", Box::new(LookupHelper {}));

        self.register_block_helper("with", Box::new(WithHelper {}));
        //self.register_helper("each", Box::new(EachHelper {}));
        //self.register_helper("if", Box::new(IfHelper {}));
        //self.register_helper("unless", Box::new(UnlessHelper {}));
    }

    /// Set the escape function for the registry.
    pub fn set_escape(&mut self, escape: EscapeFn) {
        self.escape = escape;
    }

    pub fn escape(&self) -> &EscapeFn {
        &self.escape
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

    pub fn get_helper(&self, name: &str) -> Option<&Box<dyn Helper + 'reg>> {
        self.helpers.get(name)
    }

    pub fn get_block_helper(
        &self,
        name: &str,
    ) -> Option<&Box<dyn BlockHelper + 'reg>> {
        self.block_helpers.get(name)
    }

    pub fn render<'a, T>(
        &'a self,
        templates: &'a Templates<'a>,
        name: &str,
        data: &T,
    ) -> Result<String>
    where
        T: Serialize,
    {
        let mut writer = StringOutput::new();
        self.render_to_write(templates, name, data, &mut writer)?;
        Ok(writer.into())
    }

    pub fn render_to_write<'a, T>(
        &'a self,
        templates: &'a Templates<'a>,
        name: &str,
        data: &T,
        writer: &mut impl Output,
    ) -> Result<'a, ()>
    where
        T: Serialize,
    {
        let tpl = templates
            //.templates()
            .get(name)
            .ok_or_else(|| Error::TemplateNotFound(name.to_string()))?;
        tpl.render(self, templates, name, data, writer)?;
        Ok(())
    }
}
