# Spelling Guide

This document explains how we handle spelling in the Torrust Tracker Deploy project using CSpell.

## 🎯 Overview

We use [CSpell](https://cspell.org/) for spell checking across all project files including documentation, comments, and code identifiers. This helps maintain consistency and professionalism in our codebase.

## 📋 Configuration

### CSpell Configuration

Our spell checking is configured through `cspell.json` in the project root:

```json
{
  "$schema": "https://raw.githubusercontent.com/streetsidesoftware/cspell/main/cspell.schema.json",
  "version": "0.2",
  "dictionaryDefinitions": [
    {
      "name": "project-words",
      "path": "./project-words.txt",
      "addWords": true
    }
  ],
  "dictionaries": ["project-words"],
  "ignorePaths": ["target", "/project-words.txt"]
}
```

### Project Dictionary

The `project-words.txt` file contains:

- Technical terms specific to our domain
- Proper nouns (company names, product names)
- Acronyms and abbreviations
- Valid identifiers that aren't in standard dictionaries

## 🚀 Running Spell Checks

### Via Linting System

```bash
# Run CSpell individually
cargo run --bin linter cspell

# Run all linters including CSpell
cargo run --bin linter all
```

### Direct CSpell Commands

```bash
# Check all files
cspell .

# Check with suggestions and context
cspell . --no-progress --show-suggestions --show-context

# Check specific files
cspell "src/**/*.rs" "docs/**/*.md"

# Get list of unknown words
cspell --words-only --unique .
```

## 🔧 Fixing Spelling Issues

### 1. Fix Actual Misspellings

For genuine spelling errors, fix them directly in the source files.

### 2. Add Valid Terms to Dictionary

For legitimate technical terms, proper nouns, or domain-specific vocabulary:

```bash
# Add words to project-words.txt (one per line, sorted alphabetically)
echo "kubernetes" >> project-words.txt
echo "dockerfile" >> project-words.txt
sort -u project-words.txt -o project-words.txt
```

### 3. Handle Special Cases

#### Code Identifiers

Valid variable names, function names, and identifiers should be added to the dictionary:

```text
# Examples in project-words.txt
ansible
containerd
rustfmt
shellcheck
torrust
```

#### Tokens and Keys

For security tokens, API keys, and similar strings, there are several approaches:

1. **Add to Dictionary (Recommended)**:

   ```text
   # In project-words.txt - for test/fixture tokens only
   AAAAB
   EAAAADAQABAAABAQC
   ```

2. **Use CSpell Ignore Comments**:

   ```rust
   // cspell:disable-next-line
   const API_TOKEN: &str = "abc123def456ghi789";

   /* cspell:disable */
   const COMPLEX_TOKEN: &str = "very-long-generated-token-here";
   /* cspell:enable */
   ```

3. **Configuration Patterns**:

   ```json
   {
     "ignoreRegExpList": ["/auth_token: .*/g", "/api[_-]key: .*/gi"]
   }
   ```

## 📝 Best Practices

### Do Add to Dictionary

- ✅ Technical terms: `kubernetes`, `dockerfile`, `ansible`
- ✅ Project names: `torrust`, `opentofu`
- ✅ Tool names: `rustfmt`, `shellcheck`, `yamllint`
- ✅ Domain concepts: `provisioning`, `infra`
- ✅ Test fixture data (non-sensitive)

### Don't Add to Dictionary

- ❌ Actual misspellings
- ❌ Typos in variable names
- ❌ Real secrets or production tokens
- ❌ Random strings that could mask real errors

### Guidelines for Adding Words

1. **Be Conservative**: Only add words you're confident are correct
2. **Use Lowercase**: Add words in lowercase unless they're proper nouns
3. **Check Alternatives**: Consider if there's a standard spelling first
4. **Document Context**: Add comments in `project-words.txt` for unusual terms

Example `project-words.txt` structure:

```text
# Infrastructure and DevOps
ansible
containerd
dockerfile
kubernetes
multipass
opentofu

# Project-specific terms
torrust
rustfmt
shellcheck

# Test fixtures (non-sensitive)
testkey
mocksecret
```

## 🔍 Troubleshooting

### Common Issues

1. **Too Many False Positives**

   - Review and clean up the project dictionary
   - Use more specific ignore patterns
   - Consider CSpell ignore comments for edge cases

2. **Missing Technical Terms**

   - Add them to `project-words.txt`
   - Keep them lowercase unless proper nouns
   - Sort alphabetically for maintainability

3. **Generated or Binary Content**
   - Add paths to `ignorePaths` in `cspell.json`
   - Use glob patterns to exclude file types

### Getting Help

- Check CSpell suggestions: `cspell --show-suggestions <file>`
- View CSpell documentation: [https://cspell.org/docs/](https://cspell.org/docs/)
- Review existing patterns in `cspell.json`

## 🧪 Integration with Development Workflow

### Pre-commit Checks

CSpell is included in the mandatory pre-commit checks:

```bash
cargo run --bin linter all  # Includes CSpell
```

### CI/CD Integration

The spell checker runs automatically in CI/CD pipelines as part of the linting process.

### IDE Integration

Consider installing CSpell extensions for your IDE:

- VS Code: "Code Spell Checker" extension
- Other IDEs: Check CSpell documentation for integration options

## 🎯 Goals

- **Consistency**: Maintain professional spelling across all project content
- **Quality**: Catch typos early in the development process
- **Maintainability**: Keep a clean, well-organized project dictionary
- **Developer Experience**: Provide clear guidance for handling spelling issues

By following these guidelines, we ensure that our codebase maintains high quality while accommodating the technical nature of our domain vocabulary.
