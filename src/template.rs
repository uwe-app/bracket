//! Templates add rendering capability to nodes.
use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use std::fmt;

use crate::{
    error::{Error, SyntaxError},
    escape::EscapeFn,
    helper::HelperRegistry,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    registry::Registry,
    render::Render,
    RenderResult, Result,
};

#[derive(Default)]
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

#[derive(Default)]
pub struct Templates<'source> {
    templates: HashMap<&'source str, Template<'source>>,
}

impl<'source> Templates<'source> {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn build(&mut self, loader: &'source Loader) -> Result<'source, ()> {
        for (k, v) in loader.sources() {
            let template = Templates::compile(v, Default::default()).unwrap();
            self.register(k.as_str(), template);
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        name: &'source str,
        template: Template<'source>,
    ) {
        self.templates.insert(name, template);
    }

    pub fn unregister(
        &mut self,
        name: &'source str,
    ) -> Option<Template<'source>> {
        self.templates.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&Template<'source>> {
        self.templates.get(name)
    }

    pub fn compile(s: &str, options: ParserOptions) -> Result<Template<'_>> {
        Ok(Template::compile(s, options).map_err(Error::from)?)
    }
}

#[derive(Debug)]
pub struct Template<'source> {
    source: &'source str,
    node: Node<'source>,
}

impl<'source> Template<'source> {
    pub fn new(source: &'source str, node: Node<'source>) -> Self {
        Self { source, node }
    }

    pub fn as_str(&self) -> &'source str {
        self.source
    }

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
    ) -> std::result::Result<Template, SyntaxError<'source>> {
        let mut parser = Parser::new(source, options);
        let node = parser.parse()?;
        Ok(Template::new(source, node))
    }

    /// Render this template to the given writer.
    pub fn render<T>(
        &self,
        escape: &EscapeFn,
        helpers: &'reg HelperRegistry<'reg>,
        templates: &'source Templates<'source>,
        name: &str,
        data: &T,
        writer: &mut impl Output,
    ) -> RenderResult<'_, ()>
    where
        T: Serialize,
    {
        let mut rc = Render::new(
            escape,
            helpers,
            templates,
            self.source,
            data,
            Box::new(writer),
        )?;

        rc.render_node(&self.node)
    }
}
