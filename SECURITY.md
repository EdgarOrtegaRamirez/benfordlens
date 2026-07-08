# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately:

1. **Do not** open a public GitHub issue
2. Email the maintainer at the email associated with the GitHub account
3. Or open a **private security advisory** on GitHub

## Security Measures

BenfordLens implements several security best practices:

- **Input validation**: All file paths are validated to prevent path traversal
- **No external dependencies for crypto**: We use no cryptographic libraries
- **Safe parsing**: CSV/JSON parsing uses well-tested crates (csv, serde)
- **No network access**: The CLI has no network calls
- **No runtime code evaluation**: No `eval()` or dynamic code execution

## Known Issues

None reported.
