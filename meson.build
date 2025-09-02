project('torrust-testing-infra', 'rust',
  version : '0.1.0',
  default_options : ['warning_level=3'])

# Clippy linter with comprehensive checks
run_target('clippy',
  command : [
    'env',
    'CARGO_INCREMENTAL=0',
    'cargo', 'clippy',
    '--no-deps',
    '--tests',
    '--benches', 
    '--examples',
    '--workspace',
    '--all-targets',
    '--all-features',
    '--',
    '-D', 'clippy::correctness',
    '-D', 'clippy::suspicious', 
    '-D', 'clippy::complexity',
    '-D', 'clippy::perf',
    '-D', 'clippy::style',
    '-D', 'clippy::pedantic'
  ])
