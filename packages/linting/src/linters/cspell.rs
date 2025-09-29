use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

use crate::utils::{install_npm_tool, is_command_available};

/// Run the `CSpell` spell checker linter
///
/// # Errors
///
/// Returns an error if `CSpell` is not available, cannot be installed,
/// or if the spell checking fails.
pub fn run_cspell_linter() -> Result<()> {
    // Check if cspell is installed
    if !is_command_available("cspell") {
        install_npm_tool("cspell")?;
    }

    // Run the spell checker
    info!(target: "cspell", "Running spell check on all files...");

    // Run cspell on the entire project (it will use cspell.json configuration)
    let mut cmd = Command::new("cspell");
    cmd.args([".", "--no-progress", "--show-context"]);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "cspell", "All files passed spell checking!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from cspell (it usually goes to stdout)
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        println!("💡 To fix spelling issues:");
        println!("  1. Fix actual misspellings in the files");
        println!("  2. Add technical terms/proper nouns to project-words.txt");
        println!("  3. Use cspell suggestions: cspell --show-suggestions <file>");
        println!();

        error!(target: "cspell", "Spell checking failed. Please fix the issues above.");
        Err(anyhow::anyhow!("Spell checking failed"))
    }
}
