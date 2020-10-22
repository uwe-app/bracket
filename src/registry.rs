use std::collections::HashMap;

use serde::Serialize;

use crate::{template::Template, Error, Result, error::RenderError};

pub struct Registry<'reg> {
    templates: HashMap<&'reg str, Template<'reg>>,
}

impl<'reg> Registry<'reg> {
    pub fn new() -> Self {
        Self { templates: Default::default() }
    }

    pub fn compile(s: &str) -> Result<Template> {
        Ok(Template::compile(s).map_err(Error::from)?)
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

    pub fn get_template(
        &self,
        name: &'reg str,
    ) -> Result<&Template<'reg>> {
        self.templates.get(name).ok_or_else(|| {
            Error::from(RenderError::TemplateNotFound(name.to_string()))
        })
    }

    pub fn register_template_string(
        &mut self,
        name: &'reg str,
        s: &'reg str,
    ) -> Result<()> {
        let tpl = Registry::compile(s)?;
        Ok(self.register_template(name, tpl))
    }

    pub fn render<T>(&self, name: &'reg str, data: &T) -> Result<String> 
        where T: Serialize {

        let tpl = self.get_template(name)?;
        println!("Do a render {:?}", tpl);
        
        Ok(String::new())
    }
}
