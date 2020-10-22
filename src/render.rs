use serde::Serialize;
use serde_json::Value;

use crate::{error::RenderError, output::Output, registry::Registry};

pub trait Renderer<'reg, 'render> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError>;
}

pub struct RenderState {}

impl RenderState {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct RenderContext<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    state: RenderState,
    writer: Box<&'render mut dyn Output>,
}

impl<'reg, 'render> RenderContext<'reg, 'render> {
    pub fn new<T: Serialize>(
        registry: &'reg Registry<'reg>,
        data: &T,
        state: RenderState,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError> {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        Ok(Self {
            registry,
            root,
            state,
            writer,
        })
    }

    pub fn write_str(&mut self, s: &str) -> Result<usize, RenderError> {
        Ok(self.writer.write_str(s).map_err(RenderError::from)?)
    }
}
