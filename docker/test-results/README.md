# Test Results Directory

This directory stores test results from Docker-based test runs.

## Structure

```
test-results/
├── rust/           # Rust cargo test output
├── android/        # JUnit XML from Android Gradle tests
└── integration/    # Integration test logs and results
```

## Generated Files

- `*.log` - Test execution logs
- `*.xml` - JUnit XML test results (Android)
- `*.json` - JSON-formatted test summaries
- `*.txt` - Plain text test reports

## Viewing Results

### Console Output
Test results are printed to console during test execution.

### File Results
```bash
# View all results
find . -type f

# View latest log
ls -t *.log | head -1 | xargs cat

# View Android test results
find android -name '*.xml' -exec cat {} \;
```

## Cleanup

Results are automatically created but not automatically cleaned up.

To clean:
```bash
rm -rf test-results/*
```

Or use the clean flag:
```bash
cd docker
./run-all-tests.sh --clean
```

## CI/CD

In CI/CD, these results are uploaded as artifacts and can be downloaded from:
- GitHub Actions → Workflow run → Artifacts section

## Note

This directory is gitignored - test results are not committed to the repository.
