# Contributing to OCMS

Thank you for your interest in contributing to the Open College Management System! 🎓

## How to Contribute

### Reporting Issues
- Use the [GitHub Issues](https://github.com/your-org/ocms/issues) tracker
- Search for existing issues before opening a new one
- Include reproduction steps and environment details

### Pull Requests
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit with clear messages: `git commit -m "feat: add attendance export"`
4. Push and open a PR against `main`

### Code Standards
- **Frontend**: ESLint + Prettier, TypeScript strict mode
- **Backend**: `cargo clippy --deny warnings`, `cargo fmt`
- All new features must include tests
- PRs must pass all CI checks before merging

### Branch Strategy
| Branch | Purpose |
|--------|---------|
| `main` | Production-ready code |
| `develop` | Integration branch |
| `feature/*` | New features |
| `fix/*` | Bug fixes |
| `release/*` | Release preparation |

### Commit Message Format
We follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat: add student transcript export
fix: correct attendance calculation
docs: update API reference
chore: upgrade dependencies
```

## Development Setup
See [README.md](README.md) for full setup instructions.

## Code of Conduct
All contributors must adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).
