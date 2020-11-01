//! Main entry point for compiling, storing and rendering templates.
use std::collections::HashMap;
use std::path::Path;

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
    template::Template,
    Error, Result,
};

pub struct Loader {
    sources: HashMap<String, String>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Get the map of template content.
    pub fn sources(&self) -> &HashMap<String, String> {
        &self.sources
    }

    /// Insert a named string template.
    pub fn insert<N, S>(&mut self, name: N, content: S)
    where
        N: AsRef<str>,
        S: AsRef<str>,
    {
        self.sources
            .insert(name.as_ref().to_owned(), content.as_ref().to_owned());
    }

    /// Add a named template from a file.
    pub fn add<N, P>(&mut self, name: N, file: P) -> std::io::Result<()>
    where
        N: AsRef<str>,
        P: AsRef<Path>,
    {
        let (_, content) = self.read(file)?;
        self.insert(name, &content);
        Ok(())
    }

    /// Load a file and use the file path as the template name.
    pub fn load<P: AsRef<Path>>(&mut self, file: P) -> std::io::Result<()> {
        let (name, content) = self.read(file)?;
        self.insert(name, &content);
        Ok(())
    }

    fn read<P: AsRef<Path>>(
        &mut self,
        file: P,
    ) -> std::io::Result<(String, String)> {
        let path = file.as_ref();
        let name = path.to_string_lossy().to_owned().to_string();
        let content = std::fs::read_to_string(path)?;
        Ok((name, content))
    }
}

pub struct Registry<'reg> {
    templates: HashMap<&'reg str, Template<'reg>>,
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
    escape: EscapeFn,
}

impl<'reg> Registry<'reg> {

    pub fn new() -> Self {
        let mut reg = Self {
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

    pub fn get_block_helper(
        &self,
        name: &str,
    ) -> Option<&Box<dyn BlockHelper + 'reg>> {
        self.block_helpers.get(name)
    }

    pub fn compile(
        s: &str,
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

    pub fn get_template(&self, name: &str) -> Option<&Template<'_>> {
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
        let tpl = self
            .get_template(name)
            .ok_or_else(|| Error::TemplateNotFound(name.to_string()))?;
        tpl.render(self, name, data, writer)?;
        Ok(())
    }

}
