//! - Identifying file formats based on extensions
//! - Validating template file structure
//!
//! This module is **NOT** responsible for:
//! - Template rendering or variable substitution
//! - File I/O operations
//! - Template resolution or compilation
//!
//! # Supported File Types
//!
//! - **Static files**: Direct configuration files (`.yml`, `.yaml`, `.toml`, `.tf`)
//! - **Tera templates**: Dynamic templates with variable substitution (`.yml.tera`, `.toml.tera`, `.tf.tera`)
//!
//! # Examples
//!
//! ```rust
//! use torrust_tracker_deploy::template::file::File;
//!
//! // Create DTO for static YAML file
//! let static_file = File::new("config/app.yml", "key: value".to_string())?;
//!
//! // Create DTO for Tera template with YAML output
//! let template_file = File::new("templates/inventory.yml.tera", "host: {{ vm_ip }}".to_string())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fmt::Display;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Engine {
    Static,
    Tera,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Tera,
    Yml,
    Toml,
    Tf,
    Tfvars,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Tera,
    Yaml,
    Yml,
    Toml,
    Tf,
    Tfvars,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Template file path cannot be empty")]
    EmptyPath,

    #[error("Template file path must contain a filename: {path}")]
    MissingFilename { path: String },

    #[error("Template file must have an extension: {path}")]
    MissingExtension { path: String },

    #[error("Unknown file extension '{extension}' in file: {path}")]
    UnknownExtension { path: String, extension: String },

    #[error("Tera template file must have an inner extension to determine output format: {path}")]
    MissingInnerExtension { path: String },

    #[error("Unknown inner extension '{extension}' in Tera template file: {path}")]
    UnknownInnerExtension { path: String, extension: String },

    #[error("Invalid inner extension '{extension}' for Tera template file: {path}. Tera templates cannot have 'tera' as inner extension")]
    InvalidInnerExtension { path: String, extension: String },
}

impl TryFrom<&str> for Format {
    type Error = String; // Use simple string error for Format conversion

    fn try_from(extension: &str) -> Result<Self, Self::Error> {
        match extension.to_lowercase().as_str() {
            "tera" => Ok(Format::Tera),
            "yml" | "yaml" => Ok(Format::Yml),
            "toml" => Ok(Format::Toml),
            "tf" => Ok(Format::Tf),
            _ => Err(extension.to_string()),
        }
    }
}

impl TryFrom<&str> for Extension {
    type Error = String;

    fn try_from(extension: &str) -> Result<Self, Self::Error> {
        match extension.to_lowercase().as_str() {
            "tera" => Ok(Extension::Tera),
            "yaml" => Ok(Extension::Yaml),
            "yml" => Ok(Extension::Yml),
            "toml" => Ok(Extension::Toml),
            "tf" => Ok(Extension::Tf),
            "tfvars" => Ok(Extension::Tfvars),
            _ => Err(extension.to_string()),
        }
    }
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extension::Tera => write!(f, "tera"),
            Extension::Yaml => write!(f, "yaml"),
            Extension::Yml => write!(f, "yml"),
            Extension::Toml => write!(f, "toml"),
            Extension::Tf => write!(f, "tf"),
            Extension::Tfvars => write!(f, "tfvars"),
        }
    }
}

/// A Data Transfer Object (DTO) representing template file metadata for the deployment system.
///
/// The `File` struct encapsulates metadata about template files without being responsible
/// for template resolution or rendering. It serves as a structured representation of
/// template information that can be passed between components in the deployment pipeline.
///
/// # Purpose
///
/// This DTO is designed to:
/// - Parse and store template file metadata (path, format, engine type)
/// - Provide a standardized representation of template information
/// - Enable validation of template file structure and format
/// - Facilitate template processing by other components
///
/// **Note**: This struct does NOT handle template rendering or variable resolution.
/// Those responsibilities belong to dedicated template engine components.
///
/// # Template Engine Detection
///
/// The engine is determined by the file extension pattern:
/// - Files ending with `.tera` are processed as Tera templates
/// - All other files are treated as static files
///
/// # Format Detection
///
/// For Tera templates (`.ext.tera`), the inner extension determines the output format.
/// For static files, the extension directly determines the format.
///
/// Supported formats: `yml`/`yaml`, `toml`, `tf`, `tera`
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deploy::template::file::File;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create DTO for a static YAML file
/// let static_file = File::new("config/app.yml", "key: value".to_string())?;
/// assert_eq!(static_file.engine(), &torrust_tracker_deploy::template::file::Engine::Static);
///
/// // Create DTO for a Tera template that outputs YAML
/// let template_file = File::new("templates/inventory.yml.tera", "host: {{ vm_ip }}".to_string())?;
/// assert_eq!(template_file.engine(), &torrust_tracker_deploy::template::file::Engine::Tera);
/// assert_eq!(template_file.inner_format(), Some(&torrust_tracker_deploy::template::file::Format::Yml));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct File {
    engine: Engine,

    /// The original full path of the template file, e.g., `templates/ansible/inventory.yml.tera`
    path: String,

    /// The filename without the directory path, e.g., `inventory.yml.tera`
    filename: String,

    /// The file format based on the extension, e.g., `yml`, `toml`, `tf`, `tera`
    format: Format,

    /// The file extension, e.g., `yml`, `yaml`, `toml`, `tf`, `tera`
    extension: Extension,

    /// When the file is a template, the inner format (e.g., `yml`, `toml`, `tf`)
    inner_format: Option<Format>,

    /// When the file is a template, the inner extension (e.g., `yml`, `yaml`, `toml`, `tf`)
    inner_extension: Option<Extension>,

    /// The content of the template file as a string
    content: String,
}

impl File {
    /// Creates a new template file with metadata extracted from the path
    ///
    /// # Arguments
    /// * `path` - Full path to the template file (e.g., "templates/ansible/inventory.yml.tera")
    /// * `content` - The content of the template file as a string
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file path does not contain a file extension
    /// - The file extension is not recognized
    /// - A Tera template file (.tera) does not have a valid inner extension to determine output format
    ///
    /// # Examples
    /// ```
    /// # use torrust_tracker_deploy::template::file::File;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::new("templates/ansible/inventory.yml.tera", "content here".to_string())?;
    /// // This creates a Tera template with yml inner format
    ///
    /// let static_file = File::new("templates/ansible/wait-cloud-init.yml", "---\nkey: value".to_string())?;
    /// // This creates a static yml file
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: &str, content: String) -> Result<File, Error> {
        // Check for empty path first
        if path.is_empty() {
            return Err(Error::EmptyPath);
        }

        let filename = Self::extract_filename(path);

        // Check if we actually got a filename from the path
        if filename.is_empty() {
            return Err(Error::MissingFilename {
                path: path.to_string(),
            });
        }

        let (extension, inner_extension) = Self::extract_extensions(&filename, path)?;

        let (engine, format, inner_format, final_inner_extension) = if extension == Extension::Tera
        {
            // This is a Tera template file (e.g., inventory.yml.tera)
            if let Some(ref inner_ext) = inner_extension {
                let inner_format = match inner_ext {
                    Extension::Yml | Extension::Yaml => Format::Yml,
                    Extension::Toml => Format::Toml,
                    Extension::Tf => Format::Tf,
                    Extension::Tfvars => Format::Tfvars,
                    Extension::Tera => {
                        return Err(Error::InvalidInnerExtension {
                            path: path.to_string(),
                            extension: "tera".to_string(),
                        });
                    }
                };
                (
                    Engine::Tera,
                    Format::Tera,
                    Some(inner_format),
                    inner_extension,
                )
            } else {
                // This is a .tera file with no inner extension - not allowed
                return Err(Error::MissingInnerExtension {
                    path: path.to_string(),
                });
            }
        } else {
            // This is a static file with a single extension
            let format = match extension {
                Extension::Yml | Extension::Yaml => Format::Yml,
                Extension::Toml => Format::Toml,
                Extension::Tf => Format::Tf,
                Extension::Tfvars => Format::Tfvars,
                Extension::Tera => {
                    // Single .tera extension without inner extension - not allowed
                    return Err(Error::MissingInnerExtension {
                        path: path.to_string(),
                    });
                }
            };
            (Engine::Static, format, None, None)
        };

        Ok(File {
            engine,
            path: path.to_string(),
            filename,
            format,
            extension,
            inner_format,
            inner_extension: final_inner_extension,
            content,
        })
    }

    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    #[must_use]
    pub fn format(&self) -> &Format {
        &self.format
    }

    #[must_use]
    pub fn extension(&self) -> &Extension {
        &self.extension
    }

    #[must_use]
    pub fn inner_format(&self) -> Option<&Format> {
        self.inner_format.as_ref()
    }

    #[must_use]
    pub fn inner_extension(&self) -> Option<&Extension> {
        self.inner_extension.as_ref()
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Extracts the filename from a file path
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to extract the filename from
    ///
    /// # Returns
    ///
    /// The filename as a String, or an empty string if the path is invalid
    fn extract_filename(path: &str) -> String {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string()
    }

    /// Extracts file extensions from a filename
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to extract extensions from (e.g., "inventory.yml.tera")
    /// * `path` - The full file path (for error reporting)
    ///
    /// # Returns
    ///
    /// A tuple containing the last extension and optionally the previous extension
    /// For example: "inventory.yml.tera" returns (`Extension::Tera`, `Some(Extension::Yml)`)
    /// For example: "config.yml" returns (`Extension::Yml`, `None`)
    fn extract_extensions(
        filename: &str,
        path: &str,
    ) -> Result<(Extension, Option<Extension>), Error> {
        let extensions: Vec<&str> = filename
            .split('.')
            .skip(1) // Skip the base name
            .collect();

        if extensions.is_empty() {
            return Err(Error::MissingExtension {
                path: path.to_string(),
            });
        }

        // Get the last extension (required)
        let last_extension = extensions.last().unwrap();
        let extension = Extension::try_from(*last_extension).map_err(|unknown_ext| {
            Error::UnknownExtension {
                path: path.to_string(),
                extension: unknown_ext,
            }
        })?;

        // Get the previous extension if it exists
        let inner_extension = if extensions.len() >= 2 {
            let inner_ext = extensions[extensions.len() - 2];
            Some(Extension::try_from(inner_ext).map_err(|unknown_ext| {
                Error::UnknownInnerExtension {
                    path: path.to_string(),
                    extension: unknown_ext,
                }
            })?)
        } else {
            None
        };

        Ok((extension, inner_extension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_a_static_yml_template() {
        let path = "templates/ansible/wait-cloud-init.yml";
        let content = "---
# Ansible Playbook: Wait for cloud-init completion
- name: Wait for cloud-init completion
  hosts: all
  gather_facts: false
  become: true
  tasks:
    - name: Wait for cloud-init to complete
      command: cloud-init status --wait
"
        .to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Static);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "wait-cloud-init.yml");
        assert_eq!(file.format(), &Format::Yml);
        assert_eq!(file.inner_format(), None);
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_create_a_tera_template_with_yml_inner_format() {
        let path = "templates/ansible/inventory.yml.tera";
        let content = "# Ansible Inventory File (YAML format)
all:
  hosts:
    torrust-vm:
      ansible_host: {{ vm_ip }}
      ansible_user: {{ vm_user }}
      ansible_ssh_private_key_file: {{ ssh_private_key_path }}
      ansible_ssh_common_args: '-o StrictHostKeyChecking=no'
"
        .to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "inventory.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_create_a_tera_template_with_toml_inner_format() {
        let path = "templates/config/app.toml.tera";
        let content = "[server]
host = \"{{ server_host }}\"
port = {{ server_port }}

[database]
url = \"{{ db_url }}\"
"
        .to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "app.toml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Toml));
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_create_a_static_toml_template() {
        let path = "config/app.toml";
        let content = "[server]
host = \"localhost\"
port = 8080

[database]
url = \"sqlite://db.sqlite\"
"
        .to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Static);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "app.toml");
        assert_eq!(file.format(), &Format::Toml);
        assert_eq!(file.inner_format(), None);
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_create_a_terraform_tera_template() {
        let path = "templates/tofu/main.tf.tera";
        let content = "resource \"lxd_container\" \"{{ container_name }}\" {
  name  = \"{{ container_name }}\"
  image = \"ubuntu:{{ ubuntu_version }}\"
  
  config = {
    \"user.user-data\" = file(\"{{ cloud_init_path }}\")
  }
}
"
        .to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "main.tf.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Tf));
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_convert_format_from_extension() {
        assert_eq!(Format::try_from("yml"), Ok(Format::Yml));
        assert_eq!(Format::try_from("yaml"), Ok(Format::Yml));
        assert_eq!(Format::try_from("toml"), Ok(Format::Toml));
        assert_eq!(Format::try_from("tf"), Ok(Format::Tf));
        assert_eq!(Format::try_from("tera"), Ok(Format::Tera));
        assert!(Format::try_from("unknown").is_err());
        assert_eq!(Format::try_from("unknown").unwrap_err(), "unknown");
    }

    #[test]
    fn it_should_fail_when_file_has_no_extension() {
        let path = "templates/ansible/hosts";
        let content = "localhost".to_string();

        let result = File::new(path, content);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::MissingExtension {
                path: path.to_string()
            }
        );
    }

    #[test]
    fn it_should_fail_when_file_has_unknown_extension() {
        let path = "templates/config/app.unknown";
        let content = "content".to_string();

        let result = File::new(path, content);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::UnknownExtension {
                path: path.to_string(),
                extension: "unknown".to_string()
            }
        );
    }

    #[test]
    fn it_should_handle_complex_tera_template_paths() {
        let path = "templates/deeply/nested/config.production.yml.tera";
        let content = "production: {{ is_production }}".to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), "config.production.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_fail_when_a_tera_template_does_not_have_an_inner_extension() {
        let path = "templates/config/template.tera";
        let content = "{{ some_variable }}".to_string();

        let result = File::new(path, content);

        // This should fail because we don't know what format the resolved template should be
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::MissingInnerExtension {
                path: path.to_string()
            }
        );
    }

    #[test]
    fn it_should_fail_when_tera_template_has_unknown_inner_extension() {
        let path = "templates/config/app.unknown.tera";
        let content = "{{ some_variable }}".to_string();

        let result = File::new(path, content);

        // This should fail because the inner extension is not recognized
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::UnknownInnerExtension {
                path: path.to_string(),
                extension: "unknown".to_string()
            }
        );
    }

    #[test]
    fn it_should_fail_when_tera_template_has_tera_as_inner_extension() {
        let path = "templates/config/app.tera.tera";
        let content = "{{ some_variable }}".to_string();

        let result = File::new(path, content);

        // This should fail because "tera" is not allowed as an inner extension
        // It doesn't make sense to have a .tera.tera file
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::InvalidInnerExtension {
                path: path.to_string(),
                extension: "tera".to_string()
            }
        );
    }

    #[test]
    fn it_should_fail_when_path_is_empty() {
        let path = "";
        let content = "content".to_string();

        let result = File::new(path, content);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::EmptyPath);
    }

    #[test]
    fn it_should_fail_when_directory_path_has_no_filename() {
        let path = "templates/ansible/";
        let content = "content".to_string();

        let result = File::new(path, content);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::MissingExtension {
                path: path.to_string()
            }
        );
    }

    #[test]
    fn it_should_fail_when_path_resolves_to_no_filename() {
        let path = ".";
        let content = "content".to_string();

        let result = File::new(path, content);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::MissingFilename {
                path: path.to_string()
            }
        );
    }

    #[test]
    fn it_should_handle_hidden_files_with_tera_template() {
        let path = "templates/.hidden.yml.tera";
        let content = "key: {{ value }}".to_string();

        let file = File::new(path, content.clone()).expect("Failed to create file");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.path(), path);
        assert_eq!(file.filename(), ".hidden.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
        assert_eq!(file.content(), &content);
    }

    #[test]
    fn it_should_handle_case_insensitive_extensions() {
        let path_upper = "templates/config/app.YML.TERA";
        let path_mixed = "templates/config/app.Yml.Tera";
        let content = "key: {{ value }}".to_string();

        let file_upper = File::new(path_upper, content.clone())
            .expect("Failed to create file with uppercase extensions");
        let file_mixed = File::new(path_mixed, content.clone())
            .expect("Failed to create file with mixed case extensions");

        // Both should work due to case-insensitive matching
        assert_eq!(file_upper.engine(), &Engine::Tera);
        assert_eq!(file_upper.format(), &Format::Tera);
        assert_eq!(file_upper.inner_format(), Some(&Format::Yml));

        assert_eq!(file_mixed.engine(), &Engine::Tera);
        assert_eq!(file_mixed.format(), &Format::Tera);
        assert_eq!(file_mixed.inner_format(), Some(&Format::Yml));
    }

    #[test]
    fn it_should_handle_special_characters_in_filename() {
        let path = "templates/config@2024/app-v1.2.yml.tera";
        let content = "version: {{ app_version }}".to_string();

        let file = File::new(path, content.clone())
            .expect("Failed to create file with special characters");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.filename(), "app-v1.2.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
    }

    #[test]
    fn it_should_handle_multiple_intermediate_extensions() {
        let path = "templates/config.production.staging.deployment.yml.tera";
        let content = "env: {{ environment }}".to_string();

        let file = File::new(path, content.clone())
            .expect("Failed to create file with multiple extensions");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(
            file.filename(),
            "config.production.staging.deployment.yml.tera"
        );
        assert_eq!(file.format(), &Format::Tera);
        // Should still correctly identify yml as the inner format (second-to-last extension)
        assert_eq!(file.inner_format(), Some(&Format::Yml));
    }

    #[test]
    fn it_should_handle_filename_starting_with_dot() {
        let path = ".yml";
        let content = "key: value".to_string();

        let file = File::new(path, content.clone()).expect("Should create file for .yml");

        // ".yml" is treated as a filename with extension "yml"
        assert_eq!(file.engine(), &Engine::Static);
        assert_eq!(file.filename(), ".yml");
        assert_eq!(file.format(), &Format::Yml);
        assert_eq!(file.inner_format(), None);
    }

    #[test]
    fn it_should_fail_when_filename_has_only_dots() {
        let path = "templates/...";
        let content = "content".to_string();

        let result = File::new(path, content);

        // "..." results in empty extensions, leading to UnknownExtension with empty string
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::UnknownExtension {
                path: path.to_string(),
                extension: String::new()
            }
        );
    }

    #[test]
    fn it_should_treat_yaml_and_yml_extensions_the_same() {
        let path_yaml = "templates/config.yaml";
        let path_yml = "templates/config.yml";
        let content = "key: value".to_string();

        let file_yaml = File::new(path_yaml, content.clone()).expect("Failed to create .yaml file");
        let file_yml_format =
            File::new(path_yml, content.clone()).expect("Failed to create .yml file");

        // Both yaml and yml should be treated as Format::Yml
        assert_eq!(file_yaml.format(), &Format::Yml);
        assert_eq!(file_yml_format.format(), &Format::Yml);
        assert_eq!(file_yaml.engine(), &Engine::Static);
        assert_eq!(file_yml_format.engine(), &Engine::Static);
    }

    #[test]
    fn it_should_handle_duplicate_extensions_in_tera_template() {
        let path = "templates/config.yml.yml.tera";
        let content = "key: {{ value }}".to_string();

        let file = File::new(path, content.clone())
            .expect("Failed to create file with duplicate extensions");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.format(), &Format::Tera);
        // Should use the second-to-last extension (yml) as inner format
        assert_eq!(file.inner_format(), Some(&Format::Yml));
    }

    #[test]
    fn it_should_fail_when_filename_ends_with_dot() {
        let path = "templates/config.yml.";
        let content = "key: value".to_string();

        let result = File::new(path, content);

        // Filename ending with dot results in empty last extension
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::UnknownExtension {
                path: path.to_string(),
                extension: String::new()
            }
        );
    }

    #[test]
    fn it_should_handle_very_long_extension_chains() {
        let path = "config.a.b.c.d.e.f.g.h.i.j.yml.tera";
        let content = "key: {{ value }}".to_string();

        let file = File::new(path, content.clone()).expect("Should handle long extension chains");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.format(), &Format::Tera);
        // Should still correctly identify yml as inner format
        assert_eq!(file.inner_format(), Some(&Format::Yml));
        assert_eq!(file.filename(), "config.a.b.c.d.e.f.g.h.i.j.yml.tera");
    }

    #[test]
    fn it_should_handle_unicode_filenames() {
        let path = "templates/конфиг.yml.tera";
        let content = "ключ: {{ значение }}".to_string();

        let file = File::new(path, content.clone()).expect("Should handle unicode filenames");

        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.filename(), "конфиг.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
    }

    #[test]
    fn it_should_handle_relative_path_traversal() {
        let path = "../../../dangerous.yml.tera";
        let content = "malicious: {{ code }}".to_string();

        let file = File::new(path, content.clone())
            .expect("Should handle path traversal (path parsing only)");

        // The file parsing should work regardless of path traversal
        assert_eq!(file.engine(), &Engine::Tera);
        assert_eq!(file.filename(), "dangerous.yml.tera");
        assert_eq!(file.format(), &Format::Tera);
        assert_eq!(file.inner_format(), Some(&Format::Yml));
        assert_eq!(file.path(), path); // Original path preserved
    }

    #[test]
    fn it_should_fail_when_extensions_are_single_characters() {
        let path = "templates/config.a.b";
        let content = "content".to_string();

        let result = File::new(path, content);

        // Single character extensions should fail as unknown
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::UnknownExtension {
                path: path.to_string(),
                extension: "b".to_string()
            }
        );
    }
}
