//! Templates add rendering capability to nodes.
use std::collections::HashMap;

use serde::Serialize;
use std::fmt;

use crate::{
    Registry,
    escape::EscapeFn,
    helper::HelperRegistry,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::Render,
    RenderResult, SyntaxResult,
};

use rental::rental;

/// Collection of named templates.
pub type Templates<'a> = HashMap<String, Template>;

rental! {
    mod rentals {
        use super::*;
        #[rental(covariant, debug)]
        pub struct Template {
            source: String,
            node: Node<'source>,
        }
    }
}

// SEE: https://github.com/projectfluent/fluent-rs/blob/master/fluent-bundle/src/resource.rs#L5-L14

/// Template that owns the underlying string and a corresponding document node.
#[derive(Debug)]
pub struct Template(rentals::Template);

impl Template {

    /// Compile a new template resource.
    pub fn compile(source: String, options: ParserOptions) -> SyntaxResult<Self> {
        let mut errors = None;
        let res = rentals::Template::new(source, |s| match Parser::new(s, options).parse() {
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
        registry: &'a Registry<'a>,
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
            registry,
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

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node().fmt(f)
    }
}

