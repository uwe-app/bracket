//! Templates add rendering capability to nodes.
use std::collections::HashMap;
use std::convert::TryFrom;

#[cfg(feature = "fs")]
use std::ffi::OsStr;
#[cfg(feature = "fs")]
use std::path::Path;

use serde::Serialize;
use std::fmt;

use crate::{
    error::Error,
    escape::EscapeFn,
    helper::HelperRegistry,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::Render,
    RenderResult, Result, SyntaxResult,
};

/// Storage for template sources.
#[derive(Default)]
pub struct Loader {
    sources: HashMap<String, String>,
}

impl Loader {
    /// Create an empty loader.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Get the map of template source content.
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
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
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
    ///
    /// Requires the `fs` feature.
    #[cfg(feature = "fs")]
    pub fn load<P: AsRef<Path>>(&mut self, file: P) -> std::io::Result<()> {
        let (name, content) = self.read(file)?;
        self.insert(name, &content);
        Ok(())
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
                        self.insert(name, &content);
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "fs")]
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

/// Collection of named templates.
///
/// For partials to be resolved they must exist in a collection
/// that is used during a render.
#[derive(Default)]
pub struct Templates<'source> {
    templates: HashMap<&'source str, Template<'source>>,
}

impl<'source> Templates<'source> {
    /// Create an empty templates collection.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Add a named template.
    ///
    /// If a template already exists with the given name
    /// it is overwritten.
    pub fn insert(&mut self, name: &'source str, template: Template<'source>) {
        self.templates.insert(name, template);
    }

    /// Remove a named template.
    pub fn remove(&mut self, name: &'source str) -> Option<Template<'source>> {
        self.templates.remove(name)
    }

    /// Get a named template from this collection.
    pub fn get(&self, name: &str) -> Option<&Template<'source>> {
        self.templates.get(name)
    }

    /// Compile a string to a template.
    pub fn compile(s: &str, options: ParserOptions) -> Result<Template<'_>> {
        Ok(Template::compile(s, options).map_err(Error::from)?)
    }

    /// Import all the templates from a loader.
    pub fn import(&mut self, loader: &'source Loader) -> Result<()> {
        for (k, v) in loader.sources() {
            let template =
                Templates::compile(v, ParserOptions::new(k.to_string(), 0, 0))?;
            self.insert(k.as_str(), template);
        }
        Ok(())
    }
}

impl<'source> TryFrom<&'source Loader> for Templates<'source> {
    type Error = crate::error::Error;
    fn try_from(
        loader: &'source Loader,
    ) -> std::result::Result<Self, Self::Error> {
        let mut tpl = Templates::new();
        tpl.import(loader)?;

        //for (k, v) in loader.sources() {
            //let template =
                //Templates::compile(v, ParserOptions::new(k.to_string(), 0, 0))?;
            //tpl.insert(k.as_str(), template);
        //}

        Ok(tpl)
    }
}

/// Type that adds rendering capability to a document node.
#[derive(Debug)]
pub struct Template<'source> {
    node: Node<'source>,
}

impl<'source> Template<'source> {
    /// Create a new template.
    pub(crate) fn new(node: Node<'source>) -> Self {
        Self { node }
    }

    /// The document node for the template.
    pub fn node(&self) -> &'source Node {
        &self.node
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<'reg, 'source> Template<'source> {
    /// Compile a block.
    pub fn compile(
        source: &'source str,
        options: ParserOptions,
    ) -> SyntaxResult<Template> {
        let mut parser = Parser::new(source, options);
        let node = parser.parse()?;
        Ok(Template::new(node))
    }

    /// Render this template to the given writer.
    pub(crate) fn render<'a, T>(
        &self,
        strict: bool,
        escape: &EscapeFn,
        helpers: &'reg HelperRegistry<'reg>,
        templates: &'source Templates<'source>,
        name: &str,
        data: &T,
        writer: &'a mut impl Output,
    ) -> RenderResult<()>
    where
        T: Serialize,
    {
        let mut rc = Render::new(
            strict,
            escape,
            helpers,
            templates,
            name,
            data,
            Box::new(writer),
        )?;

        rc.render(&self.node)
    }
}
