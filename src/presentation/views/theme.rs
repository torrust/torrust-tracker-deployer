//! Theme configuration for user output
//!
//! This module provides theme support for user-facing messages, allowing customization
//! of visual symbols used throughout the output.

/// Output theme controlling symbols and formatting
///
/// A theme defines the visual appearance of user-facing messages through
/// configurable symbols. Themes enable consistent styling across all output
/// and support different environments (terminals, CI/CD, accessibility needs).
///
/// # Predefined Themes
///
/// - **Emoji** (default): Unicode emoji symbols for interactive terminals
/// - **Plain**: Text labels like `[INFO]`, `[OK]` for CI/CD environments
/// - **ASCII**: Basic ASCII characters for limited terminal support
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::Theme;
///
/// // Use emoji theme (default)
/// let theme = Theme::emoji();
/// assert_eq!(theme.progress_symbol(), "⏳");
///
/// // Use plain text theme for CI/CD
/// let theme = Theme::plain();
/// assert_eq!(theme.success_symbol(), "[OK]");
///
/// // Use ASCII theme for limited terminals
/// let theme = Theme::ascii();
/// assert_eq!(theme.error_symbol(), "[x]");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct Theme {
    progress_symbol: String,
    success_symbol: String,
    warning_symbol: String,
    error_symbol: String,
}

impl Theme {
    /// Create emoji theme with Unicode symbols (default)
    ///
    /// Best for interactive terminals with good Unicode support.
    /// Uses emoji characters that are visually distinctive and widely supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::Theme;
    ///
    /// let theme = Theme::emoji();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// assert_eq!(theme.success_symbol(), "✅");
    /// assert_eq!(theme.warning_symbol(), "⚠️");
    /// assert_eq!(theme.error_symbol(), "❌");
    /// ```
    #[must_use]
    pub fn emoji() -> Self {
        Self {
            progress_symbol: "⏳".to_string(),
            success_symbol: "✅".to_string(),
            warning_symbol: "⚠️".to_string(),
            error_symbol: "❌".to_string(),
        }
    }

    /// Create plain text theme for CI/CD environments
    ///
    /// Uses text labels like `[INFO]`, `[OK]`, `[WARN]`, `[ERROR]` that work
    /// in any environment without Unicode support. Ideal for CI/CD pipelines
    /// and log aggregation systems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::Theme;
    ///
    /// let theme = Theme::plain();
    /// assert_eq!(theme.progress_symbol(), "[INFO]");
    /// assert_eq!(theme.success_symbol(), "[OK]");
    /// assert_eq!(theme.warning_symbol(), "[WARN]");
    /// assert_eq!(theme.error_symbol(), "[ERROR]");
    /// ```
    #[must_use]
    pub fn plain() -> Self {
        Self {
            progress_symbol: "[INFO]".to_string(),
            success_symbol: "[OK]".to_string(),
            warning_symbol: "[WARN]".to_string(),
            error_symbol: "[ERROR]".to_string(),
        }
    }

    /// Create ASCII-only theme using basic characters
    ///
    /// Uses simple ASCII characters that work on any terminal.
    /// Good for environments with limited character set support or
    /// when maximum compatibility is required.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::Theme;
    ///
    /// let theme = Theme::ascii();
    /// assert_eq!(theme.progress_symbol(), "=>");
    /// assert_eq!(theme.success_symbol(), "[+]");
    /// assert_eq!(theme.warning_symbol(), "[!]");
    /// assert_eq!(theme.error_symbol(), "[x]");
    /// ```
    #[must_use]
    pub fn ascii() -> Self {
        Self {
            progress_symbol: "=>".to_string(),
            success_symbol: "[+]".to_string(),
            warning_symbol: "[!]".to_string(),
            error_symbol: "[x]".to_string(),
        }
    }

    /// Get the progress symbol for this theme
    #[must_use]
    pub fn progress_symbol(&self) -> &str {
        &self.progress_symbol
    }

    /// Get the success symbol for this theme
    #[must_use]
    pub fn success_symbol(&self) -> &str {
        &self.success_symbol
    }

    /// Get the warning symbol for this theme
    #[must_use]
    pub fn warning_symbol(&self) -> &str {
        &self.warning_symbol
    }

    /// Get the error symbol for this theme
    #[must_use]
    pub fn error_symbol(&self) -> &str {
        &self.error_symbol
    }
}

impl Default for Theme {
    /// Create the default theme (emoji)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::Theme;
    ///
    /// let theme = Theme::default();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// ```
    fn default() -> Self {
        Self::emoji()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_emoji_symbols_when_using_emoji_theme() {
        let theme = Theme::emoji();

        assert_eq!(theme.progress_symbol(), "⏳");
        assert_eq!(theme.success_symbol(), "✅");
        assert_eq!(theme.warning_symbol(), "⚠️");
        assert_eq!(theme.error_symbol(), "❌");
    }

    #[test]
    fn it_should_return_text_labels_when_using_plain_theme() {
        let theme = Theme::plain();

        assert_eq!(theme.progress_symbol(), "[INFO]");
        assert_eq!(theme.success_symbol(), "[OK]");
        assert_eq!(theme.warning_symbol(), "[WARN]");
        assert_eq!(theme.error_symbol(), "[ERROR]");
    }

    #[test]
    fn it_should_return_ascii_symbols_when_using_ascii_theme() {
        let theme = Theme::ascii();

        assert_eq!(theme.progress_symbol(), "=>");
        assert_eq!(theme.success_symbol(), "[+]");
        assert_eq!(theme.warning_symbol(), "[!]");
        assert_eq!(theme.error_symbol(), "[x]");
    }

    #[test]
    fn it_should_default_to_emoji_theme_when_using_default() {
        let theme = Theme::default();

        assert_eq!(theme, Theme::emoji());
    }

    #[test]
    fn themes_should_be_cloneable() {
        let theme1 = Theme::emoji();
        let theme2 = theme1.clone();
        assert_eq!(theme1, theme2);
    }
}
