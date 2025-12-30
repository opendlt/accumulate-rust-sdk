use std::fs;
use std::path::Path;

/// Prohibited patterns that indicate incomplete or stub code
const PROHIBITED_PATTERNS: &[&str] = &[
    "TODO",
    "FIXME",
    "XXX",
    "TBD",
    "HACK",
    "unimplemented!()",
    "todo!()",
    "panic!(\"TODO\")",
    "panic!(\"FIXME\")",
    "panic!(\"Not implemented\")",
    "panic!(\"not implemented\")",
];

/// Patterns that are allowed in comments but not in code
const COMMENT_ONLY_PATTERNS: &[&str] = &[
    "TODO",
    "FIXME",
    "XXX",
    "TBD",
    "HACK",
];

/// File extensions to scan
const RUST_EXTENSIONS: &[&str] = &[".rs"];

/// Directories to scan (relative to project root)
const SCAN_DIRECTORIES: &[&str] = &["src", "examples"];

/// Directories to ignore during scanning
const IGNORE_DIRECTORIES: &[&str] = &["tests", "tooling", "target", ".git"];

#[derive(Debug)]
struct ProhibitedMatch {
    file_path: String,
    line_number: usize,
    line_content: String,
    pattern: String,
    context: MatchContext,
}

#[derive(Debug)]
enum MatchContext {
    Code,
    Comment,
    String,
}

impl ProhibitedMatch {
    fn is_violation(&self) -> bool {
        match self.context {
            MatchContext::Code => true, // All patterns prohibited in code
            MatchContext::Comment => {
                // Some patterns are allowed in comments, others are not
                match self.pattern.as_str() {
                    "unimplemented!()" | "todo!()" => true, // These should never appear even in comments
                    p if p.starts_with("panic!(") => true, // Panic with stub messages not allowed
                    _ => false, // Regular TODOs/FIXMEs allowed in comments
                }
            }
            MatchContext::String => false, // Generally allowed in string literals
        }
    }
}

/// Determine the context of a match within a line
fn determine_context(line: &str, match_pos: usize) -> MatchContext {
    let before_match = &line[..match_pos];

    // Check if we're in a string literal
    let quote_count = before_match.matches('"').count();
    let escaped_quote_count = before_match.matches("\\\"").count();
    let actual_quote_count = quote_count - escaped_quote_count;
    if actual_quote_count % 2 == 1 {
        return MatchContext::String;
    }

    // Check if we're in a comment
    if before_match.contains("//") || before_match.contains("/*") {
        return MatchContext::Comment;
    }

    // Check for line comments
    if let Some(comment_pos) = line.find("//") {
        if match_pos > comment_pos {
            return MatchContext::Comment;
        }
    }

    MatchContext::Code
}

/// Scan a single file for prohibited patterns
fn scan_file(file_path: &Path) -> Result<Vec<ProhibitedMatch>, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let mut matches = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        let line_number = line_number + 1; // 1-based line numbering

        for pattern in PROHIBITED_PATTERNS {
            if let Some(match_pos) = line.find(pattern) {
                let context = determine_context(line, match_pos);

                let prohibited_match = ProhibitedMatch {
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number,
                    line_content: line.to_string(),
                    pattern: pattern.to_string(),
                    context,
                };

                matches.push(prohibited_match);
            }
        }
    }

    Ok(matches)
}

/// Recursively scan a directory for Rust files
fn scan_directory(dir_path: &Path) -> Result<Vec<ProhibitedMatch>, Box<dyn std::error::Error>> {
    let mut all_matches = Vec::new();

    if !dir_path.exists() {
        return Ok(all_matches);
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if !IGNORE_DIRECTORIES.contains(&dir_name.as_ref()) {
                let mut dir_matches = scan_directory(&path)?;
                all_matches.append(&mut dir_matches);
            }
        } else if path.is_file() {
            if let Some(extension) = path.extension() {
                if RUST_EXTENSIONS.contains(&extension.to_string_lossy().as_ref()) {
                    let mut file_matches = scan_file(&path)?;
                    all_matches.append(&mut file_matches);
                }
            }
        }
    }

    Ok(all_matches)
}

/// Get the project root directory
fn get_project_root() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Ok(std::path::PathBuf::from(manifest_dir))
}

#[test]
fn test_no_todos_in_source_code() {
    let project_root = get_project_root().expect("Failed to get project root");
    let mut all_violations = Vec::new();
    let mut total_files_scanned = 0;

    println!("Scanning for prohibited patterns in source code...");
    println!("Project root: {}", project_root.display());

    for scan_dir in SCAN_DIRECTORIES {
        let dir_path = project_root.join(scan_dir);
        println!("Scanning directory: {}", dir_path.display());

        match scan_directory(&dir_path) {
            Ok(matches) => {
                let violations: Vec<_> = matches.into_iter()
                    .filter(|m| m.is_violation())
                    .collect();

                if !violations.is_empty() {
                    println!("Found {} violations in {}", violations.len(), scan_dir);
                    all_violations.extend(violations);
                }

                // Count files for statistics
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if entry.path().is_file() &&
                           entry.path().extension().map_or(false, |ext| RUST_EXTENSIONS.contains(&ext.to_string_lossy().as_ref())) {
                            total_files_scanned += 1;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error scanning directory {}: {}", dir_path.display(), e);
            }
        }
    }

    println!("Scan complete. Files scanned: {}", total_files_scanned);

    if !all_violations.is_empty() {
        println!("\n[FAIL] QUALITY GATE FAILED: Found {} prohibited patterns in source code", all_violations.len());
        println!("=====================================================================");

        // Group violations by file
        let mut violations_by_file: std::collections::HashMap<String, Vec<&ProhibitedMatch>> =
            std::collections::HashMap::new();

        for violation in &all_violations {
            violations_by_file.entry(violation.file_path.clone())
                .or_insert_with(Vec::new)
                .push(violation);
        }

        // Print violations organized by file
        for (file_path, violations) in violations_by_file {
            println!("\nFile: {}", file_path);
            for violation in violations {
                println!("   Line {}: {} (pattern: '{}')",
                    violation.line_number,
                    violation.line_content.trim(),
                    violation.pattern
                );
                println!("           Context: {:?}", violation.context);
            }
        }

        println!("\nTip: Fix these issues before proceeding:");
        println!("   - Replace TODO/FIXME with proper implementation");
        println!("   - Remove unimplemented!() and todo!() macros");
        println!("   - Implement proper error handling instead of panic!()");
        println!("   - Add proper documentation for complex code");

        panic!("Quality gate failed: {} prohibited patterns found", all_violations.len());
    } else {
        println!("[OK] QUALITY GATE PASSED: No prohibited patterns found in source code");
        println!("   Files scanned: {}", total_files_scanned);
        println!("   Directories: {}", SCAN_DIRECTORIES.join(", "));
    }
}

#[test]
fn test_prohibited_patterns_detection() {
    // Test that our pattern detection works correctly
    let test_cases = vec![
        ("let x = 5; // TODO: optimize this", "TODO", false),
        ("unimplemented!()", "unimplemented!()", true),
        ("todo!()", "todo!()", true),
        ("panic!(\"TODO: implement this\")", "panic!(\"TODO\")", true),
        ("let msg = \"TODO: user message\";", "TODO", false), // In string literal
        ("// This is a TODO comment", "TODO", false), // In comment
        ("/* FIXME: handle error case */", "FIXME", false), // In comment
        ("println!(\"unimplemented!()\");", "unimplemented!()", false), // In string
    ];

    for (code, pattern, should_be_violation) in test_cases {
        if let Some(match_pos) = code.find(pattern) {
            let context = determine_context(code, match_pos);
            let prohibited_match = ProhibitedMatch {
                file_path: "test.rs".to_string(),
                line_number: 1,
                line_content: code.to_string(),
                pattern: pattern.to_string(),
                context,
            };

            let is_violation = prohibited_match.is_violation();
            assert_eq!(
                is_violation,
                should_be_violation,
                "Pattern '{}' in code '{}' should be violation: {}, but got: {}",
                pattern, code, should_be_violation, is_violation
            );
        }
    }
}

#[test]
fn test_directory_scanning() {
    let project_root = get_project_root().expect("Failed to get project root");

    // Test that we can scan the src directory
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        let matches = scan_directory(&src_dir).expect("Failed to scan src directory");

        // This test just verifies the scanning works, not the content
        println!("Scanned src directory, found {} total matches", matches.len());

        // Verify we're actually finding Rust files
        assert!(src_dir.join("lib.rs").exists(), "Expected lib.rs to exist in src/");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_pattern_matching_edge_cases() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_todos.rs");

        let test_content = r#"
// This file tests edge cases for TODO detection
fn main() {
    let s = "This string contains TODO but should be ignored";
    let s2 = "Another string with FIXME in it";

    // TODO: This comment should be allowed
    /* FIXME: Block comment should be allowed */

    // Other test patterns:
    // These patterns are not prohibited when in comments
    // Some examples: print!(), assert!(), etc.
}

#[test]
fn test_function() {
    // TODO: Write this test - allowed in comment
    assert_eq!(1, 1);
}
"#;

        fs::write(&test_file, test_content).expect("Failed to write test file");

        let matches = scan_file(&test_file).expect("Failed to scan test file");
        let violations: Vec<_> = matches.into_iter().filter(|m| m.is_violation()).collect();

        // Clean up
        fs::remove_file(&test_file).ok();

        // Should find no violations in this test file
        assert_eq!(violations.len(), 0, "Expected no violations, but found: {:?}", violations);
    }
}