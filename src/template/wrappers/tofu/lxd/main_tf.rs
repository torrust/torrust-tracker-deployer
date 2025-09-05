//! Template wrapper for templates/tofu/lxd/main.tf

use crate::template::{StaticContext, TemplateRenderer};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Template wrapper for templates/tofu/lxd/main.tf (static configuration)
pub struct MainTfTemplate {
    template_path: PathBuf,
}

impl MainTfTemplate {
    #[must_use]
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }
}

impl TemplateRenderer for MainTfTemplate {
    type Context = StaticContext;

    fn template_path(&self) -> &Path {
        &self.template_path
    }

    fn required_variables(&self) -> Vec<&'static str> {
        vec![]
    }

    fn render(&self, context: &Self::Context, output_path: &Path) -> Result<()> {
        self.validate_context(context)?;
        crate::template::copy_static_file(&self.template_path, output_path)
    }

    fn validate_context(&self, _context: &Self::Context) -> Result<()> {
        Ok(())
    }
}
