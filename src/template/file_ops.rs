//! File operations for template processing
//!
//! This module provides file operations including copying and writing files
//! with automatic directory creation for template processing workflows.

use std::path::Path;
use thiserror::Error;

/// Errors that can occur during file operations
#[derive(Error, Debug)]
pub enum FileOperationError {
    /// Failed to create the output directory
    #[error("Failed to create directory: {path}")]
    DirectoryCreation { path: String },

    /// Failed to write the file to the output path
    #[error("Failed to write file to: {path}")]
    FileWrite { path: String },

    /// Failed to copy the file from source to destination
    #[error("Failed to copy file from {source_path} to {dest_path}")]
    FileCopy {
        source_path: String,
        dest_path: String,
    },
}

/// Copy a file, creating parent directories if necessary
///
/// This function copies files without template processing and creates
/// any necessary parent directories in the destination path.
///
/// # Errors
/// Returns `FileOperationError::DirectoryCreation` if the destination directory cannot be created,
/// or `FileOperationError::FileCopy` if the file cannot be copied
pub fn copy_file_with_dir_creation(
    source: &Path,
    destination: &Path,
) -> Result<(), FileOperationError> {
    // Ensure destination directory exists
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|_| FileOperationError::DirectoryCreation {
            path: parent.display().to_string(),
        })?;
    }

    std::fs::copy(source, destination).map_err(|_| FileOperationError::FileCopy {
        source_path: source.display().to_string(),
        dest_path: destination.display().to_string(),
    })?;

    Ok(())
}

/// Write content to a file, creating parent directories if necessary
///
/// # Errors
/// Returns `FileOperationError::DirectoryCreation` if the parent directory cannot be created,
/// or `FileOperationError::FileWrite` if the file cannot be written
pub fn write_file_with_dir_creation(
    output_path: &Path,
    content: &str,
) -> Result<(), FileOperationError> {
    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| FileOperationError::DirectoryCreation {
            path: parent.display().to_string(),
        })?;
    }

    // Write the content to the file
    std::fs::write(output_path, content).map_err(|_| FileOperationError::FileWrite {
        path: output_path.display().to_string(),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    mod copy_file_with_dir_creation {
        use super::*;

        #[test]
        fn it_should_copy_file_to_existing_directory() {
            let temp_dir = TempDir::new().unwrap();
            let source_file = temp_dir.path().join("source.txt");
            let dest_file = temp_dir.path().join("dest.txt");

            fs::write(&source_file, "test content").unwrap();

            let result = copy_file_with_dir_creation(&source_file, &dest_file);

            assert!(result.is_ok());
            assert!(dest_file.exists());
            let content = fs::read_to_string(&dest_file).unwrap();
            assert_eq!(content, "test content");
        }

        #[test]
        fn it_should_copy_file_and_create_parent_directories() {
            let temp_dir = TempDir::new().unwrap();
            let source_file = temp_dir.path().join("source.txt");
            let dest_file = temp_dir.path().join("deep/nested/path/dest.txt");

            fs::write(&source_file, "nested test").unwrap();

            let result = copy_file_with_dir_creation(&source_file, &dest_file);

            assert!(result.is_ok());
            assert!(dest_file.exists());
            let content = fs::read_to_string(&dest_file).unwrap();
            assert_eq!(content, "nested test");
        }

        #[test]
        fn it_should_fail_when_source_file_does_not_exist() {
            let temp_dir = TempDir::new().unwrap();
            let source_file = temp_dir.path().join("nonexistent.txt");
            let dest_file = temp_dir.path().join("dest.txt");

            let result = copy_file_with_dir_creation(&source_file, &dest_file);

            assert!(result.is_err());
            match result.unwrap_err() {
                FileOperationError::FileCopy {
                    source_path,
                    dest_path,
                } => {
                    assert!(source_path.contains("nonexistent.txt"));
                    assert!(dest_path.contains("dest.txt"));
                }
                FileOperationError::DirectoryCreation { .. } => {
                    panic!("Expected FileCopy error, got DirectoryCreation")
                }
                FileOperationError::FileWrite { .. } => {
                    panic!("Expected FileCopy error, got FileWrite")
                }
            }
        }

        #[test]
        fn it_should_overwrite_existing_destination_file() {
            let temp_dir = TempDir::new().unwrap();
            let source_file = temp_dir.path().join("source.txt");
            let dest_file = temp_dir.path().join("dest.txt");

            fs::write(&source_file, "new content").unwrap();
            fs::write(&dest_file, "old content").unwrap();

            let result = copy_file_with_dir_creation(&source_file, &dest_file);

            assert!(result.is_ok());
            let content = fs::read_to_string(&dest_file).unwrap();
            assert_eq!(content, "new content");
        }

        #[test]
        fn it_should_copy_binary_file_correctly() {
            let temp_dir = TempDir::new().unwrap();
            let source_file = temp_dir.path().join("binary.bin");
            let dest_file = temp_dir.path().join("copied.bin");

            let binary_data = vec![0x00, 0x01, 0xFF, 0x7F, 0x80];
            fs::write(&source_file, &binary_data).unwrap();

            let result = copy_file_with_dir_creation(&source_file, &dest_file);

            assert!(result.is_ok());
            let copied_data = fs::read(&dest_file).unwrap();
            assert_eq!(copied_data, binary_data);
        }
    }

    mod write_file_with_dir_creation {
        use super::*;

        #[test]
        fn it_should_write_content_to_existing_directory() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            let content = "hello world";

            let result = write_file_with_dir_creation(&file_path, content);

            assert!(result.is_ok());
            assert!(file_path.exists());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, content);
        }

        #[test]
        fn it_should_write_file_and_create_parent_directories() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("deep/nested/structure/file.txt");
            let content = "nested content";

            let result = write_file_with_dir_creation(&file_path, content);

            assert!(result.is_ok());
            assert!(file_path.exists());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, content);
        }

        #[test]
        fn it_should_overwrite_existing_file() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("existing.txt");

            fs::write(&file_path, "original content").unwrap();

            let new_content = "updated content";
            let result = write_file_with_dir_creation(&file_path, new_content);

            assert!(result.is_ok());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, new_content);
        }

        #[test]
        fn it_should_handle_empty_content() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("empty.txt");

            let result = write_file_with_dir_creation(&file_path, "");

            assert!(result.is_ok());
            assert!(file_path.exists());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, "");
        }

        #[test]
        fn it_should_handle_unicode_content() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("unicode.txt");
            let content = "Hello 世界! 🚀 Émojis and spëcial chars";

            let result = write_file_with_dir_creation(&file_path, content);

            assert!(result.is_ok());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, content);
        }

        #[test]
        fn it_should_handle_multiline_content() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("multiline.txt");
            let content = "Line 1\nLine 2\nLine 3\n\nLine 5";

            let result = write_file_with_dir_creation(&file_path, content);

            assert!(result.is_ok());
            let read_content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(read_content, content);
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn it_should_display_directory_creation_error_correctly() {
            let error = FileOperationError::DirectoryCreation {
                path: "/some/path".to_string(),
            };
            let error_string = format!("{error}");
            assert!(error_string.contains("Failed to create directory"));
            assert!(error_string.contains("/some/path"));
        }

        #[test]
        fn it_should_display_file_write_error_correctly() {
            let error = FileOperationError::FileWrite {
                path: "/output/file.txt".to_string(),
            };
            let error_string = format!("{error}");
            assert!(error_string.contains("Failed to write file to"));
            assert!(error_string.contains("/output/file.txt"));
        }

        #[test]
        fn it_should_display_file_copy_error_correctly() {
            let error = FileOperationError::FileCopy {
                source_path: "/source/file.txt".to_string(),
                dest_path: "/dest/file.txt".to_string(),
            };
            let error_string = format!("{error}");
            assert!(error_string.contains("Failed to copy file from"));
            assert!(error_string.contains("/source/file.txt"));
            assert!(error_string.contains("/dest/file.txt"));
        }

        #[test]
        fn it_should_support_debug_formatting_for_errors() {
            let write_error = FileOperationError::FileWrite {
                path: "/test/path".to_string(),
            };
            let copy_error = FileOperationError::DirectoryCreation {
                path: "/test/dir".to_string(),
            };
            let file_copy_error = FileOperationError::FileCopy {
                source_path: "/src/file".to_string(),
                dest_path: "/dst/file".to_string(),
            };

            let write_debug = format!("{write_error:?}");
            let copy_debug = format!("{copy_error:?}");
            let file_copy_debug = format!("{file_copy_error:?}");

            assert!(write_debug.contains("FileWrite"));
            assert!(copy_debug.contains("DirectoryCreation"));
            assert!(file_copy_debug.contains("FileCopy"));
        }
    }
}
