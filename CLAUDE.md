# CodeGraph

## Current Focus
- **Status**: All specs complete (136/136 tasks done)
- **Next**: Production deployment, documentation polish

## Board: 0 backlog | 0 todo | 0 in progress | 0 review | 136 done

## Project Overview

Sistema enterprise que extrai entidades ontológicas de snippets UI (Material UI, Tailwind, Chakra, código proprietário) e gera código web vanilla completo a partir de queries em linguagem natural. O diferencial é a arquitetura "neural proposes, symbolic disposes" usando NARS (Non-Axiomatic Reasoning System) para raciocínio simbólico com truth-values evidenciais, mitigando alucinações de LLMs.

## Tech Stack

- **Backend**: Rust 1.83+ / Axum 0.8
- **Graph DB**: Neo4j 5.x
- **Vector DB**: Qdrant 1.12+
- **Cache**: Redis 7.x
- **Frontend**: HTMX 2.0 / TailwindCSS 4.0
- **Reasoning**: OpenNARS for Applications (ONA)

## Key Differentiators

| Feature | Description | Benefit |
|---------|-------------|---------|
| NARS Reasoning | LLM translates NL↔Narsese, NARS reasons with truth-values | Mitigates hallucinations |
| AIKR Native | Assumption of Insufficient Knowledge and Resources | One-shot learning |
| RLKGF | Graph as reward model (OpenReview 2025) | 10x cheaper than RLHF |
| Feedback Propagation | Confidence propagates via SIMILAR_TO/CAN_REPLACE | Continuous learning |
| Hybrid Retrieval | Fulltext + Vector + Pattern + NARS Inference | 91% precision vs 34% |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CodeGraph                                │
├─────────────────────────────────────────────────────────────────┤
│  MCP Server        │  REST API         │  Web Dashboard         │
│  (Claude Code)     │  (Axum)           │  (HTMX)                │
├─────────────────────────────────────────────────────────────────┤
│                     Code Generation                              │
│              (Template + NARS Reasoning)                        │
├─────────────────────────────────────────────────────────────────┤
│  Hybrid Retrieval  │  RLKGF Feedback   │  Benchmark Suite       │
│  (Vector+Graph)    │  (Confidence)     │  (Comparison)          │
├─────────────────────────────────────────────────────────────────┤
│  Graph Storage     │  Vector Storage   │  NARS Integration      │
│  (Neo4j)           │  (Qdrant+Redis)   │  (OpenNARS)            │
└─────────────────────────────────────────────────────────────────┘
```

## Workflow Guidelines

**Task Management with CWA:**

```bash
# View board
cwa task board

# Move task through workflow
cwa task move <task-id> in_progress
cwa task move <task-id> review
cwa task move <task-id> done

# Create new task
cwa task create "Task title" --spec <spec-id>
```

## Crates

| Crate | Purpose |
|-------|---------|
| `codegraph-core` | Domain entities, config, error tracking, retry policies, degradation |
| `codegraph-extraction` | AST parsing with tree-sitter, ontology mapping |
| `codegraph-graph` | Neo4j repository, schema, relations |
| `codegraph-vector` | Qdrant embeddings, Redis cache |
| `codegraph-reasoning` | NARS/ONA integration, Narsese translation |
| `codegraph-retrieval` | Hybrid search (vector + graph + NARS) |
| `codegraph-feedback` | RLKGF feedback loop, confidence propagation |
| `codegraph-generation` | GPT-4o code generation, template engine |
| `codegraph-web` | Axum API, HTMX dashboard, Prometheus metrics |
| `codegraph-mcp` | MCP server for Claude Code |
| `codegraph-benchmark` | Comparison suite, reports |
| `codegraph-cli` | Command line interface |

## Production Features

### Implemented

- **Rate Limiting**: 100 req/min per IP with Redis
- **Request Tracing**: UUID trace_id in all requests
- **Prometheus Metrics**: `/metrics` endpoint
- **Error Tracking**: Sentry-compatible abstraction
- **Retry Policies**: Exponential backoff with circuit breaker
- **Graceful Degradation**: Cached responses when services offline
- **Health Checks**: `/health`, `/health/ready`, `/health/live`

### Configuration

All config via environment variables. See README.md for full list.

## Commands

```bash
# Development
cargo build
cargo test
cargo check

# Run server
cargo run -p codegraph-cli -- serve

# Run benchmarks
cargo run -p codegraph-cli -- benchmark

# Docker
docker compose -f .cwa/docker/docker-compose.yml up -d
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/snippets` | POST | Upload code snippet |
| `/api/snippets` | GET | List snippets (paginated) |
| `/api/snippets/:id` | GET | Get snippet by ID |
| `/api/snippets/:id` | DELETE | Delete snippet |
| `/api/query` | GET | Search components |
| `/api/generate` | POST | Generate UI code |
| `/api/feedback` | POST | Submit feedback |
| `/health` | GET | Basic health check |
| `/health/ready` | GET | Readiness with service status |
| `/metrics` | GET | Prometheus metrics |
