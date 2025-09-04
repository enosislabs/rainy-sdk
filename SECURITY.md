# Security Policy

## Supported Versions

We take security seriously and actively maintain the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in the Rainy SDK, please help us by reporting it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by emailing:
- **Email**: security@enosislabs.com
- **Subject**: `[SECURITY] Vulnerability Report - Rainy SDK`

### What to Include

When reporting a security vulnerability, please include:

1. **Description**: A clear description of the vulnerability
2. **Impact**: What an attacker could achieve by exploiting this vulnerability
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Proof of Concept**: If possible, include a proof of concept
5. **Environment**: Your environment details (OS, Rust version, etc.)
6. **Contact Information**: How we can reach you for follow-up questions

### Our Response Process

1. **Acknowledgment**: We'll acknowledge receipt of your report within 48 hours
2. **Investigation**: We'll investigate the issue and determine its severity
3. **Updates**: We'll provide regular updates on our progress (at least weekly)
4. **Fix**: We'll develop and test a fix
5. **Disclosure**: We'll coordinate disclosure with you
6. **Public Release**: We'll release the fix and publish a security advisory

### Security Updates

Security updates will be released as patch versions (e.g., 0.1.1, 0.1.2) with:
- A security advisory on GitHub
- Release notes describing the vulnerability and fix
- Updated dependencies if applicable

### Responsible Disclosure

We kindly ask that you:
- Give us reasonable time to fix the issue before public disclosure
- Avoid accessing or modifying user data
- Avoid disrupting our services
- Respect the privacy of our users

### Recognition

We appreciate security researchers who help keep our users safe. With your permission, we'll acknowledge your contribution in our security advisory.

## Security Best Practices

When using the Rainy SDK, follow these security best practices:

### API Key Management

- **Never commit API keys** to version control
- **Use environment variables** for API key storage
- **Rotate keys regularly** for production applications
- **Use the minimum required permissions** for your use case

### Authentication

```rust
// Secure authentication example
use rainy_sdk::{RainyClient, AuthConfig};
use std::env;

let client = RainyClient::new(
    AuthConfig::new()
        .with_api_key(env::var("RAINY_API_KEY")?)
        .with_base_url("https://your-secure-api.vercel.app")
)?;
```

### Network Security

- **Use HTTPS** for all API communications
- **Validate SSL certificates** in production
- **Implement rate limiting** in your application
- **Monitor API usage** for suspicious activity

### Error Handling

```rust
// Secure error handling example
match client.create_chat_completion(request).await {
    Ok(response) => {
        // Handle success
        println!("Response: {}", response.choices[0].message.content);
    }
    Err(e) => {
        // Log error securely (never expose sensitive information)
        log::error!("Chat completion failed: {}", e);
        // Return generic error to user
        return Err("Request failed. Please try again later.".into());
    }
}
```

## Known Security Considerations

### Current Limitations

- **API Key Storage**: API keys are stored in memory during client lifetime
- **Network Interception**: HTTPS traffic could be intercepted with compromised certificates
- **Rate Limiting**: Client-side rate limiting may not prevent all abuse scenarios

### Future Security Enhancements

We're continuously working to improve security:

- **Token-based authentication** support
- **OAuth 2.0 integration** for enhanced security
- **Client-side encryption** for sensitive data
- **Security audit reports** and compliance certifications

## Contact

For security-related questions or concerns:
- **Security Issues**: security@enosislabs.com
- **General Support**: hello@enosislabs.com
- **Documentation**: [docs.rs/rainy-sdk](https://docs.rs/rainy-sdk)

Thank you for helping keep the Rainy SDK and its users secure! 🔒
