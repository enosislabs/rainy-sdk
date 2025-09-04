# 🤝 Contributing to Rainy SDK

Thank you for your interest in contributing to the Rainy SDK! We welcome contributions from everyone. This document provides guidelines and information for contributors.

## 📋 Table of Contents

- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Code of Conduct](#code-of-conduct)

## 🚀 Development Setup

### Prerequisites

- **Rust**: Install the latest stable version from [rustup.rs](https://rustup.rs/)
- **Git**: Version control system
- **Optional**: IDE with Rust support (VS Code with rust-analyzer, CLion, etc.)

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/enosislabs/rainy-sdk.git
cd rainy-sdk
```

### 2. Set Up Development Environment

```bash
# Install development dependencies
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-tarpaulin  # For coverage reports

# Verify installation
cargo --version
rustc --version
```

### 3. Run Initial Setup

```bash
# Build the project
cargo build

# Run tests to ensure everything works
cargo test

# Generate documentation
cargo doc --open
```

### 4. Environment Configuration

Create a `.env` file for testing (not committed to git):

```bash
# Copy example environment file
cp .env.example .env

# Edit with your test credentials
RAINY_API_KEY=your-test-api-key
```

## 🔄 Development Workflow

### 1. Choose an Issue

- Check [open issues](https://github.com/enosislabs/rainy-sdk/issues) for tasks
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Feature Branch

```bash
# Create and switch to a feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description
```

### 3. Make Changes

```bash
# Make your changes following the coding standards
# Add tests for new functionality
# Update documentation as needed

# Run tests frequently
cargo test

# Run linting
cargo clippy

# Format code
cargo fmt
```

### 4. Commit Changes

```bash
# Stage your changes
git add .

# Commit with descriptive message
git commit -m "feat: add new chat completion streaming support

- Add streaming parameter to ChatCompletionRequest
- Implement streaming response handling
- Add example for streaming usage
- Update documentation

Closes #123"
```

### 5. Commit Sign-Off

All contributions to this repository must include a Developer Certificate of Origin (DCO) sign-off. This is a lightweight way to certify that you have the right to submit the code you're contributing.

#### Why is sign-off required?

The sign-off serves several important purposes:
- **Legal compliance**: It certifies that you have the legal right to submit the code
- **Chain of custody**: It helps track who contributed what and when
- **Intellectual property**: It ensures proper attribution and licensing
- **Open source standards**: It follows industry best practices for contribution tracking

#### How to sign off your commits

**For command line users:**
Simply add the `-s` flag when committing:

```bash
git commit -s -m "Your commit message"
```

This automatically adds a sign-off line at the end of your commit message:
```
Signed-off-by: Your Name <your.email@example.com>
```

**For GitHub web interface users:**
GitHub makes this process easy by providing a sign-off option in the commit interface. When creating or editing files through GitHub's web interface, you'll see a checkbox for "Sign off and commit changes" - simply check this box before committing.

#### Verifying your sign-off

You can verify that your commits are properly signed off by checking the commit log:

```bash
git log --show-signature
```

Or to see just the sign-off lines:
```bash
git log --pretty=format:"%H %s%n%b%n" | grep "Signed-off-by"
```

### 6. Push and Create Pull Request

```bash
# Push your branch
git push origin feature/your-feature-name

# Create a Pull Request on GitHub
```

## 💻 Coding Standards

### Rust Style Guidelines

We follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/) and use `rustfmt` for formatting.

#### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt --check
```

#### Linting

```bash
# Run clippy for additional linting
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix
```

### Naming Conventions

- **Functions**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Documentation

- **Public APIs**: Must have documentation comments (`///`)
- **Complex logic**: Include inline comments explaining the "why"
- **Examples**: Provide code examples in documentation

```rust
/// Creates a new chat completion request.
///
/// # Examples
///
/// ```rust
/// use rainy_sdk::{ChatCompletionRequest, ChatMessage, ChatRole};
///
/// let request = ChatCompletionRequest::new(
///     "gpt-4",
///     vec![ChatMessage {
///         role: ChatRole::User,
///         content: "Hello!".to_string(),
///     }]
/// );
/// ```
```

### Error Handling

- Use the `RainyError` type for all public APIs
- Provide meaningful error messages
- Use `Result<T, RainyError>` for fallible operations
- Handle all `Result` values appropriately

### Async Code

- Use `async fn` for asynchronous functions
- Prefer `tokio::spawn` for concurrent tasks
- Use `async_trait` for trait methods when needed
- Handle cancellation properly with `CancellationToken`

## 🧪 Testing

### Test Structure

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test API interactions (in `tests/` directory)
- **Documentation tests**: Examples in documentation comments

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test

# Generate coverage report (requires tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            role: ChatRole::User,
            content: "Hello".to_string(),
        };

        assert_eq!(message.role, ChatRole::User);
        assert_eq!(message.content, "Hello");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = some_async_function().await;
        assert!(result.is_ok());
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use rainy_sdk::{AuthConfig, RainyClient};

#[tokio::test]
async fn test_health_check() {
    let client = RainyClient::new(
        AuthConfig::new()
            .with_api_key(std::env::var("RAINY_API_KEY").unwrap())
    ).unwrap();

    let health = client.health_check().await.unwrap();
    assert!(matches!(health.status, HealthStatus::Healthy));
}
```

### Test Coverage

We aim for high test coverage:

- **Core functionality**: >90% coverage
- **Error handling**: Test all error paths
- **Edge cases**: Test boundary conditions
- **API compatibility**: Test against real API when possible

## 📝 Pull Request Process

### Before Submitting

1. **Update documentation**: Ensure README and docs reflect your changes
2. **Add tests**: Include tests for new functionality
3. **Update CHANGELOG**: Add entry for user-facing changes
4. **Run full test suite**: `cargo test && cargo clippy && cargo fmt --check`

### PR Template

Use this template when creating a Pull Request:

```markdown
## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Refactoring
- [ ] Performance improvement

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated
- [ ] Tests added for new functionality
- [ ] CHANGELOG updated (if applicable)
- [ ] Commit messages follow conventional commits
```

### Review Process

1. **Automated Checks**: CI will run tests, linting, and formatting checks
2. **Code Review**: At least one maintainer will review your code
3. **Approval**: PR must be approved before merging
4. **Merge**: Use "Squash and merge" for clean commit history

## 🐛 Issue Reporting

### Bug Reports

When reporting bugs, please include:

- **Clear title**: Summarize the issue
- **Description**: Detailed explanation of the problem
- **Steps to reproduce**: Step-by-step instructions
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, Rust version, SDK version
- **Code sample**: Minimal code to reproduce the issue

### Feature Requests

For new features, please include:

- **Use case**: Why do you need this feature?
- **Proposed solution**: How should it work?
- **Alternatives**: Other approaches considered
- **Additional context**: Screenshots, examples, etc.

### Security Issues

- **DO NOT** report security vulnerabilities in public issues
- Email security@enosislabs.com instead
- Include detailed reproduction steps and potential impact

## 📜 Code of Conduct

### Our Standards

We are committed to providing a welcoming and inclusive environment for all contributors. We expect all participants to:

- **Be respectful**: Treat everyone with respect and kindness
- **Be collaborative**: Work together constructively
- **Be inclusive**: Welcome people from all backgrounds
- **Be patient**: Understand that everyone has different experience levels
- **Be constructive**: Focus on solutions, not blame

### Unacceptable Behavior

- Harassment or discrimination
- Offensive comments or language
- Personal attacks
- Trolling or disruptive behavior
- Publishing private information

### Enforcement

Violations of the code of conduct may result in:
- Warning from maintainers
- Temporary ban from contributing
- Permanent ban in severe cases

## 🙏 Recognition

Contributors will be recognized in:
- **CHANGELOG.md**: For all releases
- **README.md**: Notable contributors section
- **GitHub releases**: Contributor mentions

## 📞 Getting Help

- **Documentation**: [docs.rs/rainy-sdk](https://docs.rs/rainy-sdk)
- **Discussions**: [GitHub Discussions](https://github.com/enosislabs/rainy-sdk/discussions)
- **Issues**: [GitHub Issues](https://github.com/enosislabs/rainy-sdk/issues)

Thank you for contributing to the Rainy SDK! 🚀
