//! Templates add rendering capability to nodes.
use std::collections::HashMap;

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

/// Collection of named templates.
///
/// For partials to be resolved they must exist in a collection
/// that is used during a render.
#[derive(Default)]
pub struct Templates<'source> {
    templates: HashMap<String, Template<'source>>,
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
    pub fn insert(&mut self, name: &str, template: Template<'source>) {
        self.templates.insert(name.to_owned(), template);
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
