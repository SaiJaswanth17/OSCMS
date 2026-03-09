# Security Policy

## Reporting Vulnerabilities

Please **do NOT** open public GitHub issues for security vulnerabilities.

Instead, report them via email:
**security@ocms.dev** (or via GitHub private security advisories)

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (optional)

We will acknowledge reports within **48 hours** and aim to release a fix within **14 days** for critical issues.

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | ✅ Yes |
| < 1.0   | ❌ No |

## Security Features

- JWT authentication with short-lived access tokens
- Argon2id password hashing
- Role-based access control (RBAC) on every endpoint
- Input sanitization and validation
- Rate limiting on all sensitive endpoints
- Audit logging for data access
- HTTPS enforced in production
- Secure HTTP headers (CSP, HSTS, X-Frame-Options)
