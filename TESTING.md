# Testing Guide for Soteric

This guide explains how to run and write tests for the Soteric project.

## Running Tests

### Run all tests
```bash
cargo test
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run specific test module
```bash
cargo test encrypter::tests
cargo test models::tests
cargo test cli::tests
cargo test process_scan::tests
```

### Run a specific test
```bash
cargo test test_encrypt_decrypt_roundtrip
```

### Run tests with multiple threads (or single-threaded)
```bash
cargo test -- --test-threads=1
```

## Test Coverage

### Encrypter Module (`src/encrypter.rs`)
- **test_encrypt_decrypt_roundtrip**: Verifies encryption and decryption returns original data
- **test_decrypt_with_wrong_key_fails**: Ensures wrong keys cannot decrypt data
- **test_encrypt_empty_file**: Tests handling of empty files
- **test_encrypt_multiple_files**: Tests batch encryption/decryption
- **test_normalize_key_consistency**: Verifies key derivation is deterministic
- **test_normalize_key_different_salts**: Confirms different salts produce different keys
- **test_decrypt_corrupted_file_fails**: Validates error handling for corrupted files

### Models Module (`src/models.rs`)
- **test_profile_state_empty**: Tests ProfileState initialization
- **test_profile_creation**: Validates Profile struct creation
- **test_profile_state_with_data**: Tests adding profiles to state
- **test_profile_state_process_mapping**: Tests process-to-profile mappings
- **test_stored_profile_serialization**: Validates JSON serialization/deserialization

### CLI Module (`src/cli.rs`)
- **test_cli_parsing_add_profile**: Tests argument parsing for `add-profile`
- **test_cli_parsing_activate**: Tests argument parsing for `activate`
- **test_cli_parsing_scan**: Tests argument parsing for `scan`
- **test_cli_parsing_status**: Tests argument parsing for `status`
- **test_cli_parsing_set_mapping**: Tests argument parsing for `set-mapping`

### Process Scan Module (`src/process_scan.rs`)
- **detects_supported_agent_binaries**: Validates detection of AI tool keywords
- **normalizes_binary_names_from_paths**: Tests path parsing and normalization
- **parses_unix_process_output_and_deduplicates**: Tests Unix `ps` output parsing
- **display_name_prefers_binary_basename**: Validates process name formatting

## Writing New Tests

### Test Structure

Tests use the standard Rust testing conventions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Arrange
        let input = setup_data();
        
        // Act
        let result = my_function(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Using Temporary Directories

For file-based tests, use the `tempfile` crate:

```rust
use tempfile::TempDir;

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    // Use test_file for testing
}
```

### Common Assertions

```rust
assert!(condition);                    // Panics if false
assert_eq!(actual, expected);          // Panics if not equal
assert_ne!(actual, unexpected);        // Panics if equal
```

## Continuous Integration

Tests should pass before submitting PR:
```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Troubleshooting

### Tests fail with "File not found"
- Ensure temporary files are created in the temp directory, not the project root
- Use `tempfile::TempDir` for automatic cleanup

### Encryption tests fail intermittently
- Verify system entropy is available (should not be an issue on modern systems)
- Check that no other process is modifying test files

### Process scanning tests behave differently on different platforms
- Tests are platform-specific; Unix tests won't run on Windows
- Some tests use mocked data, not actual process listings

## Contributing Test Cases

When adding new features:
1. Write tests alongside implementation
2. Aim for >80% code coverage
3. Test both happy path and error cases
4. Document complex test scenarios with comments
5. Use descriptive test names that explain what is being tested
