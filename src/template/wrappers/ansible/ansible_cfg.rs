//! Template wrapper for templates/ansible/ansible.cfg

use crate::template::{StaticContext, TemplateRenderer};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Template wrapper for templates/ansible/ansible.cfg (static file)
pub struct AnsibleCfgTemplate {
    template_path: PathBuf,
}

impl AnsibleCfgTemplate {
    #[must_use]
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }
}

impl TemplateRenderer for AnsibleCfgTemplate {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ansible_cfg_template() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_file = temp_dir.path().join("ansible.cfg");
        let output_file = temp_dir.path().join("output.cfg");

        fs::write(&template_file, "[defaults]\nhost_key_checking = False")?;

        let template = AnsibleCfgTemplate::new(template_file);
        let ctx = StaticContext::default();

        template.render(&ctx, &output_file)?;

        let content = fs::read_to_string(&output_file)?;
        assert_eq!(content, "[defaults]\nhost_key_checking = False");

        Ok(())
    }
}
