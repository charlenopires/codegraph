# CodeGraph

## Current Focus
- **Status**: All specs complete (136/136 tasks done)
- **Next**: Production deployment, documentation polish

## Board: 0 backlog | 0 todo | 0 in progress | 0 review | 136 done

## Project Overview

CodeGraph is an enterprise-grade system that extracts ontological entities from UI code snippets (Material UI, Tailwind CSS, Chakra UI, Bootstrap, and proprietary code) and generates complete vanilla web code from natural language queries. The key innovation is the "neural proposes, symbolic disposes" architecture using NARS (Non-Axiomatic Reasoning System) for symbolic reasoning with evidential truth-values, effectively mitigating LLM hallucinations.

## Key Differentiators

| Feature | Description | Benefit |
|---------|-------------|---------|
| NARS Reasoning | LLM translates NL↔Narsese, NARS reasons with truth-values | Mitigates hallucinations |
| AIKR Native | Assumption of Insufficient Knowledge and Resources | One-shot learning |
| RLKGF | Graph as reward model (OpenReview 2025) | 10x cheaper than RLHF |
| Feedback Propagation | Confidence propagates via SIMILAR_TO/CAN_REPLACE | Continuous learning |
| Hybrid Retrieval | Vector + Graph + NARS Inference | 91% precision vs 34% |

## Tech Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Backend | Rust 1.83+ / Axum 0.8 | High-performance async web framework |
| Graph DB | Neo4j 5.x | Component relationships and ontology |
| Vector DB | Qdrant 1.12+ | Semantic similarity search |
| Cache | Redis 7.x | Rate limiting, response caching |
| Frontend | HTMX 2.0 / TailwindCSS 4.0 | Hypermedia-driven dashboard |
| Reasoning | OpenNARS for Applications | Symbolic reasoning with truth-values |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CodeGraph                                │
├─────────────────────────────────────────────────────────────────┤
│  MCP Server          │  REST API           │  Web Dashboard     │
│  (Claude Code)       │  (Axum)             │  (HTMX)            │
├─────────────────────────────────────────────────────────────────┤
│                     Code Generation                              │
│              (Template + NARS Reasoning)                        │
├─────────────────────────────────────────────────────────────────┤
│  Hybrid Retrieval    │  RLKGF Feedback     │  Benchmark Suite   │
│  (Vector+Graph+NARS) │  (Confidence Loop)  │  (Comparison)      │
├─────────────────────────────────────────────────────────────────┤
│  Graph Storage       │  Vector Storage     │  NARS Integration  │
│  (Neo4j)             │  (Qdrant + Redis)   │  (OpenNARS)        │
├─────────────────────────────────────────────────────────────────┤
│                    Extraction Pipeline                           │
│              (HTML/CSS/JS → Knowledge Graph)                    │
└─────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Purpose |
|-------|---------|
| `codegraph-core` | Domain entities, configuration, error handling, retry policies |
| `codegraph-extraction` | AST parsing with tree-sitter, ontology mapping |
| `codegraph-graph` | Neo4j repository, schema definitions, relationship management |
| `codegraph-vector` | Qdrant embeddings, Redis caching layer |
| `codegraph-reasoning` | NARS/ONA integration, Narsese translation |
| `codegraph-retrieval` | Hybrid search combining vector, graph, and NARS inference |
| `codegraph-feedback` | RLKGF feedback loop, confidence propagation |
| `codegraph-generation` | GPT-4o code generation, template engine |
| `codegraph-web` | Axum REST API, HTMX dashboard, Prometheus metrics |
| `codegraph-mcp` | MCP server for Claude Code integration |
| `codegraph-benchmark` | Comparison suite for evaluating retrieval approaches |
| `codegraph-cli` | Command-line interface |

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

## Commands

```bash
# Development
cargo build
cargo test
cargo check
cargo clippy
cargo fmt

# Run server
cargo run -p codegraph-cli -- serve

# Run benchmarks
cargo run -p codegraph-cli -- benchmark

# Docker infrastructure
docker compose -f .cwa/docker/docker-compose.yml up -d

# Run without ONA (offline mode)
CODEGRAPH_ONA_ENABLED=false cargo run -p codegraph-cli -- serve
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/snippets` | POST | Upload a code snippet |
| `/api/snippets` | GET | List snippets (paginated) |
| `/api/snippets/:id` | GET | Get snippet by ID |
| `/api/snippets/:id` | DELETE | Delete a snippet |
| `/api/query` | GET | Search components with `?q=` parameter |
| `/api/generate` | POST | Generate UI code from description |
| `/api/feedback` | POST | Submit RLKGF feedback |
| `/api/stats` | GET | Knowledge graph statistics |
| `/api/metrics/rlkgf` | GET | RLKGF metrics and trends |
| `/health` | GET | Basic health check |
| `/health/ready` | GET | Readiness check with service status |
| `/health/live` | GET | Liveness probe |
| `/metrics` | GET | Prometheus metrics |

## MCP Tools

| Tool | Description |
|------|-------------|
| `extract_snippet` | Extract UI elements from HTML/CSS/JS code |
| `query_ui` | Search components using NARS reasoning |
| `generate_code` | Generate UI code from natural language |
| `give_feedback` | Provide RLKGF feedback (thumbs up/down) |
| `get_graph_stats` | Get knowledge graph statistics |

## Production Features

- **Rate Limiting**: 100 req/min per IP with Redis
- **Request Tracing**: UUID trace_id in all requests
- **Prometheus Metrics**: `/metrics` endpoint for monitoring
- **Error Tracking**: Sentry-compatible abstraction
- **Retry Policies**: Exponential backoff with circuit breaker
- **Graceful Degradation**: Cached responses when services offline
- **Health Checks**: `/health`, `/health/ready`, `/health/live`

### Degradation Modes

| Mode | Condition | Behavior |
|------|-----------|----------|
| Normal | All services operational | Full functionality |
| Degraded | Non-critical services offline (Redis, ONA) | Core features work |
| Cached | Critical services offline | Serves cached responses only |
| Offline | Degradation disabled and services down | System unavailable |

## Configuration

All configuration via environment variables. Key settings:

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_PORT` | `3000` | Server port |
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j connection |
| `QDRANT_URL` | `http://localhost:6334` | Qdrant URL |
| `REDIS_URL` | - | Redis URL (optional) |
| `OPENAI_API_KEY` | - | OpenAI API key (required) |
| `CODEGRAPH_ONA_ENABLED` | `true` | Enable NARS reasoning |
| `LOG_LEVEL` | `info` | Log level |

See README.md for the complete configuration reference.

## Supported Design Systems

- Material UI
- Tailwind CSS
- Chakra UI
- Bootstrap
- Custom (auto-detected)
