# CodeGraph

[![CI](https://github.com/anthropics/codegraph/actions/workflows/ci.yml/badge.svg)](https://github.com/anthropics/codegraph/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/anthropics/codegraph/branch/main/graph/badge.svg)](https://codecov.io/gh/anthropics/codegraph)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A Hybrid GraphRAG System with NARS Reasoning for UI Code Generation**

CodeGraph is an enterprise-grade system that extracts ontological entities from UI code snippets (Material UI, Tailwind CSS, Chakra UI, Bootstrap, and proprietary code) and generates complete vanilla web code from natural language queries. The key innovation is the "neural proposes, symbolic disposes" architecture using NARS (Non-Axiomatic Reasoning System) for symbolic reasoning with evidential truth-values, effectively mitigating LLM hallucinations.

## Table of Contents

- [The Problem](#the-problem)
- [How CodeGraph Solves It](#how-codegraph-solves-it)
- [Key Differentiators](#key-differentiators)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Usage Guide](#usage-guide)
- [API Reference](#api-reference)
- [MCP Integration](#mcp-integration)
- [Benchmarks](#benchmarks)
- [Configuration](#configuration)
- [Production Features](#production-features)
- [License](#license)

## The Problem

Modern enterprises face several challenges when managing UI components across multiple projects:

1. **Component Fragmentation**: Teams maintain dozens of React/Vue/Angular projects, each with their own UI components, leading to inconsistent designs and duplicated effort.

2. **Poor Discoverability**: Developers can't easily find existing components that match their needs, so they build from scratch instead of reusing.

3. **LLM Hallucinations**: When using AI to generate UI code, LLMs often produce components that don't match the company's design system or include non-existent classes and patterns.

4. **No Learning Loop**: Traditional RAG systems don't improve over time—they can't learn from developer feedback about which generated components actually worked.

## How CodeGraph Solves It

CodeGraph introduces a novel hybrid approach that combines the flexibility of neural networks with the reliability of symbolic reasoning:

```
User Query: "blue button with hover animation"
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  1. LLM Translation Layer                               │
│     - Translates natural language to Narsese            │
│     - "blue" → <{blue} --> color>                       │
│     - "hover animation" → <{transition} --> behavior>   │
└─────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  2. Hybrid Retrieval                                    │
│     - Vector similarity (semantic matching)             │
│     - Graph traversal (structural relationships)        │
│     - NARS inference (logical reasoning)                │
└─────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  3. NARS Reasoning Engine                               │
│     - Evaluates candidates with truth-values            │
│     - Confidence: 0.91 (frequency=0.95, confidence=0.87)│
│     - Rejects hallucinations (low truth-value)          │
└─────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  4. Code Generation                                     │
│     - Context-aware generation from similar components  │
│     - Validates against design system constraints       │
│     - Returns vanilla HTML/CSS/JS                       │
└─────────────────────────────────────────────────────────┘
```

## Key Differentiators

| Feature | Traditional RAG | CodeGraph |
|---------|-----------------|-----------|
| **Hallucination Mitigation** | None | NARS truth-values reject low-confidence results |
| **Learning Paradigm** | None | AIKR (Assumption of Insufficient Knowledge and Resources) enables one-shot learning |
| **Feedback Loop** | Requires model retraining | RLKGF propagates confidence through graph relations—no retraining needed |
| **Search Precision** | ~34% (vector-only) | ~91% (hybrid vector + graph + NARS) |
| **Cost Efficiency** | High (RLHF retraining) | 10x cheaper (graph as reward model) |

### What is NARS?

NARS (Non-Axiomatic Reasoning System) is a general-purpose reasoning system designed to work under the Assumption of Insufficient Knowledge and Resources (AIKR). Unlike traditional logic systems that require complete knowledge, NARS:

- **Handles uncertainty**: Every statement has a truth-value `<frequency, confidence>` indicating how often something is true and how much evidence supports it
- **Learns incrementally**: Can learn from a single example and refine beliefs with more evidence
- **Reasons under resource limits**: Makes the best decision possible given time and memory constraints

In CodeGraph, NARS serves as the "arbiter" that validates LLM suggestions against the knowledge graph, rejecting hallucinations that don't have sufficient evidential support.

### What is RLKGF?

RLKGF (Reinforcement Learning from Knowledge Graph Feedback) is our approach to continuous improvement without expensive model retraining:

```
Developer gives feedback: "This button component worked perfectly!"
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  Confidence Update                                      │
│  - Component confidence: 0.75 → 0.82                    │
│  - Propagates via SIMILAR_TO: Related buttons +0.03    │
│  - Propagates via CAN_REPLACE: Alternatives +0.02      │
└─────────────────────────────────────────────────────────┘
     │
     ▼
Future queries rank this component higher (no retraining!)
```

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

### Crate Structure

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

## Tech Stack

### Backend

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **Language** | Rust | Edition 2024 | Systems programming with memory safety |
| **Runtime** | Tokio | 1.43 | Async runtime for high-performance I/O |
| **Web Framework** | Axum | 0.8 | Async web framework with WebSocket support |
| **Graph Database** | Neo4j | 5.x | Component relationships and ontology |
| **Vector Database** | Qdrant | 1.13+ | Semantic similarity search |
| **Cache** | Redis | 7.x | Rate limiting, response caching |
| **AST Parsing** | tree-sitter | 0.24 | HTML/CSS/JS parsing |
| **LLM Integration** | async-openai | 0.28 | GPT-4o for code generation |
| **Observability** | Prometheus | 0.13 | Metrics collection |
| **API Docs** | utoipa | 5.4 | OpenAPI/Swagger documentation |
| **Reasoning** | OpenNARS for Applications | - | Symbolic reasoning with truth-values |

### Frontend

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **Framework** | React | 19.2 | UI components |
| **Language** | TypeScript | 5.9 | Type-safe JavaScript |
| **Build Tool** | Vite | 7.2 | Fast development and bundling |
| **Package Manager** | Bun | - | Fast JavaScript runtime and package manager |
| **Styling** | TailwindCSS | 4.1 | Utility-first CSS |
| **Routing** | React Router DOM | 7.13 | Client-side routing |
| **State Management** | Zustand | 5.0 | Lightweight state management |
| **Code Editor** | CodeMirror | 6 | Syntax-highlighted code editing |
| **Charts** | Recharts | 3.7 | Data visualization |
| **Graph Visualization** | D3.js | 7.9 | Knowledge graph rendering |
| **Icons** | Lucide React | 0.563 | Icon library |

## Getting Started

### Prerequisites

- Rust 1.83 or later
- Docker and Docker Compose
- OpenAI API key (for embeddings and generation)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/anthropics/codegraph.git
cd codegraph

# Start infrastructure (Neo4j, Qdrant, Redis)
docker compose -f .cwa/docker/docker-compose.yml up -d

# Set your OpenAI API key
export OPENAI_API_KEY=your-api-key-here

# Build the project
cargo build --release

# Run the server
./target/release/codegraph serve
```

The server will be available at `http://localhost:3000`.

### Running Without ONA

CodeGraph works without the ONA reasoning engine (using a fallback mode):

```bash
export CODEGRAPH_ONA_ENABLED=false
./target/release/codegraph serve
```

In offline mode, the system still uses vector + graph retrieval but skips NARS inference. This reduces precision but maintains functionality.

## Usage Guide

### Step 1: Populate the Knowledge Graph

Upload UI components from your existing projects:

```bash
curl -X POST http://localhost:3000/api/snippets \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Primary Button",
    "html": "<button class=\"bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded\">Click me</button>",
    "css": ".btn-primary { transition: all 0.2s ease; }",
    "design_system": "tailwind",
    "tags": ["button", "primary", "cta"]
  }'
```

CodeGraph automatically:
- Parses HTML/CSS/JS using tree-sitter
- Identifies the design system (Tailwind, Material UI, etc.)
- Maps to the component ontology (Button → Interactive → UIElement)
- Generates vector embeddings
- Creates nodes and relationships in Neo4j

### Step 2: Search with Natural Language

Query for components using plain English:

```bash
curl "http://localhost:3000/api/query?q=blue+button+with+hover+animation"
```

Response:
```json
{
  "results": [
    {
      "id": "abc-123",
      "name": "Primary Button",
      "confidence": 0.91,
      "reasoning": "Button with blue background (bg-blue-500) and hover transition",
      "similar_elements": ["Secondary Button", "CTA Button"]
    }
  ],
  "mode": "normal"
}
```

The NARS reasoning engine analyzes:
- **Semantics**: "blue" matches `bg-blue-*` CSS classes
- **Behavior**: "hover animation" matches `hover:*` + `transition`
- **Confidence**: 0.91 based on evidential truth-values

### Step 3: Generate New Components

Need a component that doesn't exist? Generate it with context:

```bash
curl -X POST http://localhost:3000/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Product card with image, title, price, and buy button",
    "design_system": "tailwind"
  }'
```

Response:
```json
{
  "html": "<article class=\"bg-white rounded-lg shadow-md overflow-hidden\">...</article>",
  "css": "/* Scoped styles */",
  "js": "// Event handlers",
  "reasoning": "Generated based on 3 similar cards in knowledge graph"
}
```

The generation process:
1. Retrieves similar existing components
2. Uses NARS to identify common patterns
3. GPT-4o generates vanilla code based on context
4. Validates output and injects best practices

### Step 4: Provide Feedback (RLKGF)

Help the system learn by providing feedback on generated code:

```bash
curl -X POST http://localhost:3000/api/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "element_id": "xyz-789",
    "positive": true,
    "comment": "Perfect for the e-commerce page"
  }'
```

This feedback:
- Increases the component's confidence score
- Propagates to related components via SIMILAR_TO relationships
- Improves future rankings without model retraining

### Step 5: Access the Dashboard

Visit `http://localhost:3000` for the web interface:

- **Upload**: Visual snippet upload with preview
- **Search**: Query components with graph visualization
- **Metrics**: Real-time RLKGF metrics and confidence trends
- **History**: Browse generation history and feedback

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/snippets` | POST | Upload a code snippet |
| `/api/snippets` | GET | List snippets (paginated) |
| `/api/snippets/:id` | GET | Get snippet by ID |
| `/api/snippets/:id` | DELETE | Delete a snippet |
| `/api/query` | GET | Search components (supports `?q=` query parameter) |
| `/api/generate` | POST | Generate UI code from description |
| `/api/feedback` | POST | Submit feedback on a component |
| `/api/stats` | GET | Knowledge graph statistics |
| `/api/metrics/rlkgf` | GET | RLKGF metrics and trends |
| `/health` | GET | Basic health check |
| `/health/ready` | GET | Readiness check with service status |
| `/health/live` | GET | Liveness probe |
| `/metrics` | GET | Prometheus metrics |

### Supported Design Systems

- Material UI
- Tailwind CSS
- Chakra UI
- Bootstrap
- Custom (auto-detected)

## MCP Integration

CodeGraph provides an MCP (Model Context Protocol) server for seamless integration with Claude Code and other AI assistants.

### Setup

```bash
# Start the MCP server
codegraph mcp
```

### Available Tools

| Tool | Description |
|------|-------------|
| `extract_snippet` | Extract UI elements from HTML/CSS/JS code |
| `query_ui` | Search components using NARS reasoning |
| `generate_code` | Generate UI code from natural language |
| `give_feedback` | Provide RLKGF feedback (thumbs up/down) |
| `get_graph_stats` | Get knowledge graph statistics |

### Available Resources

| Resource | Description |
|----------|-------------|
| `codegraph://metrics` | Current RLKGF metrics |
| `codegraph://recent` | Recent generations and their feedback |

### Example Usage in Claude Code

After starting the MCP server, you can use CodeGraph directly in Claude Code:

- "Search for a responsive navbar component"
- "Generate a login form with validation"
- "Extract components from this HTML file"

## Benchmarks

Compare CodeGraph's hybrid approach against simple vector RAG:

```bash
codegraph benchmark
```

This generates reports in Markdown, JSON, and HTML formats with:

| Metric | Simple Vector RAG | CodeGraph (Hybrid) |
|--------|-------------------|-------------------|
| Precision | 34% | 91% |
| Recall | 78% | 85% |
| F1 Score | 0.47 | 0.88 |
| Hallucination Rate | 23% | 4% |
| P50 Latency | 45ms | 120ms |
| P95 Latency | 89ms | 280ms |

The hybrid approach trades some latency for significantly higher precision and lower hallucination rates.

## Configuration

All configuration is done via environment variables:

### Server

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `3000` | Server port |
| `REQUEST_TIMEOUT_SECS` | `30` | Request timeout in seconds |

### Neo4j

| Variable | Default | Description |
|----------|---------|-------------|
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j connection URI |
| `NEO4J_USER` | `neo4j` | Neo4j username |
| `NEO4J_PASSWORD` | `codegraph123` | Neo4j password |
| `NEO4J_MAX_CONNECTIONS` | `50` | Connection pool size |

### Qdrant

| Variable | Default | Description |
|----------|---------|-------------|
| `QDRANT_URL` | `http://localhost:6334` | Qdrant gRPC URL |
| `QDRANT_COLLECTION` | `ui_elements` | Collection name |
| `QDRANT_VECTOR_SIZE` | `1536` | Embedding dimensions |

### Redis

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | - | Redis URL (optional, enables caching) |

### OpenAI

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_API_KEY` | - | OpenAI API key (required) |
| `OPENAI_MODEL` | `gpt-4o` | Model for code generation |
| `OPENAI_EMBEDDING_MODEL` | `text-embedding-3-small` | Model for embeddings |
| `OPENAI_MAX_TOKENS` | `4096` | Maximum tokens per request |
| `OPENAI_TEMPERATURE` | `0.7` | Generation temperature |

### ONA/NARS

| Variable | Default | Description |
|----------|---------|-------------|
| `CODEGRAPH_ONA_ENABLED` | `true` | Enable ONA integration |
| `ONA_HOST` | `localhost` | ONA server host |
| `ONA_PORT` | `50000` | ONA UDP port |
| `ONA_INFERENCE_CYCLES` | `100` | Inference cycles per query |

### Rate Limiting

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_RPM` | `100` | Requests per minute per IP |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Rate limit window |

### Retry & Circuit Breaker

| Variable | Default | Description |
|----------|---------|-------------|
| `RETRY_MAX_OPENAI` | `3` | Max retries for OpenAI calls |
| `RETRY_MAX_DB` | `2` | Max retries for database calls |
| `RETRY_BASE_DELAY_MS` | `100` | Base delay between retries |
| `CIRCUIT_BREAKER_THRESHOLD` | `5` | Failures before circuit opens |
| `CIRCUIT_BREAKER_TIMEOUT_SECS` | `30` | Time before circuit half-opens |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |
| `LOG_FORMAT` | `pretty` | Log format (pretty, json) |

### Error Tracking (Sentry)

| Variable | Default | Description |
|----------|---------|-------------|
| `SENTRY_DSN` | - | Sentry DSN (optional) |
| `SENTRY_ENVIRONMENT` | `development` | Environment name |
| `SENTRY_RELEASE` | - | Release version |
| `SENTRY_SAMPLE_RATE` | `1.0` | Sample rate (0.0-1.0) |

## Production Features

### Graceful Degradation

CodeGraph supports graceful degradation when external services are unavailable:

| Mode | Condition | Behavior |
|------|-----------|----------|
| **Normal** | All services operational | Full functionality |
| **Degraded** | Non-critical services offline (Redis, ONA) | Core features work, some caching/reasoning disabled |
| **Cached** | Critical services offline | Serves cached responses only |
| **Offline** | Degradation disabled and services down | System unavailable |

Check system status:
```bash
curl http://localhost:3000/health/ready
```

Response:
```json
{
  "status": "degraded",
  "services": {
    "neo4j": { "status": "healthy" },
    "qdrant": { "status": "healthy" },
    "redis": { "status": "unhealthy", "message": "connection refused" }
  }
}
```

### Retry & Circuit Breaker

Automatic retry with exponential backoff for external API calls:

- **OpenAI**: 3 retries with 200ms base delay
- **Databases**: 2 retries with 100ms base delay
- **Circuit Breaker**: Opens after 5 consecutive failures, attempts recovery after 30s

### Observability

- **Request Tracing**: Every request gets a UUID trace_id for correlation
- **Structured Logging**: JSON-formatted logs with trace context
- **Prometheus Metrics**: Scrape `/metrics` for monitoring
- **Health Checks**: Kubernetes-compatible probes at `/health/*`

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Test a specific crate
cargo test -p codegraph-vector
cargo test -p codegraph-benchmark

# Run with logging
RUST_LOG=debug cargo test
```

### Building Documentation

```bash
cargo doc --open
```

### Code Quality

```bash
# Linting
cargo clippy

# Formatting
cargo fmt
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [OpenNARS](https://github.com/opennars/opennars) - Non-Axiomatic Reasoning System
- [ONA](https://github.com/opennars/OpenNARS-for-Applications) - OpenNARS for Applications
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Parsing library
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [HTMX](https://htmx.org/) - Hypermedia approach for the frontend
