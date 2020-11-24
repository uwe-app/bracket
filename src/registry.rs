//! Primary entry point for compiling and rendering templates.
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "fs")]
use std::ffi::OsStr;
#[cfg(feature = "fs")]
use std::path::Path;

use crate::{
    escape::{self, EscapeFn},
    helper::HelperRegistry,
    output::{Output, StringOutput},
    parser::{Parser, ParserOptions},
    template::{Template, Templates},
    Error, Result,
};

/// Registry is the entry point for compiling and rendering templates.
///
/// A template name is always required for error messages.
pub struct Registry<'reg, 'source> {
    sources: HashMap<String, String>,
    helpers: HelperRegistry<'reg>,
    templates: Arc<RwLock<Templates<'source>>>,
    escape: EscapeFn,
    strict: bool,
}

impl<'reg, 'source> Registry<'reg, 'source> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            helpers: HelperRegistry::new(),
            templates: Arc::new(RwLock::new(Default::default())),
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

    /// Templates collection.
    fn templates(&self) -> &Arc<RwLock<Templates<'source>>> {
        &self.templates
    }

    /// Insert a named string template.
    pub fn insert<N>(&mut self, name: N, content: String)
    where
        N: AsRef<str>,
    {
        self.sources.insert(name.as_ref().to_owned(), content);
    }

    /// Add a named template from a file.
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn add<P>(&mut self, name: String, file: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        let (_, content) = self.read(file)?;
        self.sources.insert(name, content);
        Ok(())
    }

    /// Load a file and use the file path as the template name.
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn load<P: AsRef<Path>>(&mut self, file: P) -> Result<String> {
        let (name, content) = self.read(file)?;
        self.sources.insert(name.clone(), content);
        Ok(name)
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
    ) -> std::io::Result<()> {
        let ext = OsStr::new(extension);
        for entry in std::fs::read_dir(file.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == ext {
                        let name = path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_owned()
                            .to_string();
                        let (_, content) = self.read(path)?;
                        self.sources.insert(name, content);
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

    /// Compile all the loaded sources into templates.
    pub fn build<'a: 'source>(&'a self) -> Result<()> {
        let mut templates = self.templates.write().unwrap();
        for (k, v) in &self.sources {
            let template = Template::compile(v, ParserOptions::new(k.to_string(), 0, 0))?;
            templates.insert(k, template);
        }
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
    pub fn once<T>(&self, name: &str, source: &'source str, data: &T) -> Result<String>
    where
        T: Serialize,
    {
        let templates = self.templates().read().unwrap();

        let mut writer = StringOutput::new();
        let template =
            self.compile(source, ParserOptions::new(name.to_string(), 0, 0))?;
        template.render(
            self.strict(),
            self.escape(),
            self.helpers(),
            &*templates,
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
        let templates = self.templates().read().unwrap();
        let mut writer = StringOutput::new();
        template.render(
            self.strict(),
            self.escape(),
            self.helpers(),
            &*templates,
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

        let templates = self.templates().read().unwrap();
        let tpl = templates
            .get(name)
            .ok_or_else(|| Error::TemplateNotFound(name.to_string()))?;
        tpl.render(
            self.strict(),
            self.escape(),
            self.helpers(),
            &*templates,
            name,
            data,
            writer,
        )?;
        
        Ok(())
    }
}
