# 🎓 OCMS — Open College Management System

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-15+-black.svg)](https://nextjs.org/)
[![Bun](https://img.shields.io/badge/Bun-latest-pink.svg)](https://bun.sh/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-blue.svg)](https://www.postgresql.org/)

> A modern, scalable, open-source College Management System designed for any educational institution.

---

## ✨ Features

- 🔐 **Multi-Role Auth** — Students, Faculty, Admins, Department Heads
- 🎓 **Full Academic Lifecycle** — Admissions → Enrollment → Exams → Graduation
- 📊 **Analytics Dashboards** — Role-specific insights and KPIs
- 🎨 **Dynamic Theming** — Light/dark mode + custom institution branding
- 🏛️ **Multi-Institution Ready** — Deploy for any college
- 🔒 **Privacy-First** — RBAC, audit logs, encrypted data
- 📡 **Observable** — Prometheus, Grafana, Loki, Jaeger

---

## 🏗️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Next.js 15 (App Router) + Bun + TypeScript + SCSS |
| Backend | Rust + Axum + Tokio |
| Database | Supabase (PostgreSQL) + Prisma ORM |
| Auth | Custom JWT + Argon2 |
| DevOps | Docker Compose + GitHub Actions |
| Observability | Prometheus + Grafana + Loki + Jaeger |

---

## 🚀 Quickstart

### Prerequisites

- [Bun](https://bun.sh/) >= 1.0
- [Rust](https://www.rust-lang.org/) >= 1.75 (2021 edition)
- [Docker & Docker Compose](https://www.docker.com/)

### Development

```bash
# Clone the repository
git clone https://github.com/your-org/ocms.git
cd ocms

# Copy environment files
cp .env.example .env

# Start all services (DB, backend, frontend, monitoring)
docker-compose up --build

# In separate terminals for hot reload:
# Frontend
cd frontend && bun install && bun run dev

# Backend
cd backend && cargo run
```

Frontend: http://localhost:3000  
Backend API: http://localhost:8080  
Grafana: http://localhost:3001  
Jaeger: http://localhost:16686

---

## 📁 Project Structure

```
ocms/
├── frontend/          # Next.js + Bun (TypeScript + SCSS)
├── backend/           # Rust + Axum API server
├── prisma/            # Prisma schema & migrations
├── docker/            # Service-specific Docker files
├── .github/workflows/ # CI/CD pipelines
├── docs/              # Documentation
└── docker-compose.yml
```

---

## 📖 Documentation

- [Architecture Overview](docs/architecture.md)
- [API Reference](docs/api.md)
- [Deployment Guide](docs/deployment.md)
- [Contributing Guide](CONTRIBUTING.md)

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

---

## 🛡️ Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

---

## 📄 License

MIT — see [LICENSE](LICENSE).
