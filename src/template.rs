//! Templates add rendering capability to nodes.
use std::collections::HashMap;

use serde::Serialize;
use std::fmt;

use std::borrow::Cow;

use crate::{
    escape::EscapeFn,
    helper::HelperRegistry,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::Render,
    RenderResult, SyntaxResult,
};

use rental::rental;

rental! {
    mod rentals {
        use super::*;
        #[rental(covariant, debug)]
        pub struct TemplateResource {
            source: String,
            node: Node<'source>,
        }
    }
}

/// Template resource is a template that owns the underlying string.
#[derive(Debug)]
pub struct TemplateResource(rentals::TemplateResource);

impl TemplateResource {

    /// Compile a new template resource.
    pub fn compile(source: String, options: ParserOptions) -> SyntaxResult<Self> {
        let mut errors = None;
        let res = rentals::TemplateResource::new(source, |s| match Parser::new(s, options).parse() {
            Ok(ast) => ast,
            Err(err) => {
                errors = Some(err);
                Default::default()
            }
        });

        if let Some(errors) = errors {
            Err(errors)
        } else {
            Ok(Self(res))
        }
    }

    /// The document node for the template.
    pub fn node(&self) -> &Node<'_> {
        self.0.all().node
    }

    /// Render this template to the given writer.
    pub(crate) fn render<'a, T>(
        &self,
        strict: bool,
        escape: &EscapeFn,
        helpers: &'a HelperRegistry<'a>,
        templates: &'a Templates<'a>,
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

        rc.render(self.node())
    }
}

impl fmt::Display for TemplateResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node().fmt(f)
    }
}

/// Collection of named templates.
pub type Templates<'a> = HashMap<String, Template<'a>>;

/// Type that adds rendering capability to a document node.
#[derive(Debug, Default)]
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

    /// Compile a block.
    pub fn compile<'a>(
        source: &'a str,
        options: ParserOptions,
    ) -> SyntaxResult<Template<'a>> {
        let mut parser = Parser::new(source, options);
        let node = parser.parse()?;
        Ok(Template::new(node))
    }

    /// Render this template to the given writer.
    pub(crate) fn render<'a, T>(
        &self,
        strict: bool,
        escape: &EscapeFn,
        helpers: &'a HelperRegistry<'a>,
        templates: &'a Templates<'a>,
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

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

