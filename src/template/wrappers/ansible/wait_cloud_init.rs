//! Template wrapper for templates/ansible/wait-cloud-init.yml

use crate::template::{StaticContext, TemplateRenderer};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Template wrapper for templates/ansible/wait-cloud-init.yml (static playbook)
pub struct WaitCloudInitTemplate {
    template_path: PathBuf,
}

impl WaitCloudInitTemplate {
    #[must_use]
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }
}

impl TemplateRenderer for WaitCloudInitTemplate {
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
