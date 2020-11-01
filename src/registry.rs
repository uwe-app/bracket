//! Main entry point for compiling, storing and rendering templates.
use std::path::Path;
use std::collections::HashMap;

use serde::Serialize;

use crate::{
    error::RenderError,
    escape::{html_escape, EscapeFn},
    helper::{
        //EachHelper, Helper, IfHelper, LookupHelper, UnlessHelper,
        //WithHelper,
        JsonHelper,
        Helper,
        BlockHelper,
        WithHelper
    },
    output::{Output, StringOutput},
    parser::ParserOptions,
    template::Template,
    log::LogHelper,
    Error, Result,
};

pub struct Registry<'reg> {
    files: HashMap<String, String>,
    templates: HashMap<&'reg str, Template<'reg>>,
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
    escape: EscapeFn,
}

impl<'reg, 'source> Registry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            files: HashMap::new(),
            templates: Default::default(),
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

    pub fn get_block_helper(&self, name: &str) -> Option<&Box<dyn BlockHelper + 'reg>> {
        self.block_helpers.get(name)
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

    pub fn get_template(&self, name: &'reg str) -> Option<&Template<'reg>> {
        self.templates.get(name)
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

    fn register_compiled_template(
        registry: &'reg mut Registry<'reg>,
        name: &'reg str,
        template: Template<'reg>,
    ) {
        registry.templates.insert(name, template);
    }

    fn load_file<P: AsRef<Path>>(
        registry: &'reg mut Registry<'reg>,
        file: P) -> Result<'reg, &'reg str> {

        let path = file.as_ref();
        let file_name = path.to_string_lossy().to_owned().to_string();
        let content = std::fs::read_to_string(path)?;
        registry.files.insert(file_name.clone(), content);
        Ok(registry.files.get(&file_name).unwrap().as_str())
    }

    /// Register a file as a template.
    ///
    /// If a file with the same path already exists it is overwritten.
    pub fn register_template_file<P: AsRef<Path>>(
        &mut self,
        name: &'reg str,
        file: P,
    ) -> Result<()> {
        let path = file.as_ref();
        let file_name = path.to_string_lossy().to_owned().to_string();
        //let content = std::fs::read_to_string(path)?;
        //self.files.insert(file_name.clone(), content);

        /*
        let source = Registry::load_file(self, file)?;
        let options = ParserOptions { file_name, line_offset: 0, byte_offset: 0 };
        let tpl = Registry::compile(source, options)?;
        Ok(Registry::register_compiled_template(self, name, tpl))
        */
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
        let tpl = self.get_template(name).ok_or_else(|| {
            Error::TemplateNotFound(name.to_string())
        })?;
        tpl.render(self, name, data, writer)?;
        Ok(())
    }
}
