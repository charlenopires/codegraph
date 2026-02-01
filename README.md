# CodeGraph

[![CI](https://github.com/anthropics/codegraph/actions/workflows/ci.yml/badge.svg)](https://github.com/anthropics/codegraph/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/anthropics/codegraph/branch/main/graph/badge.svg)](https://codecov.io/gh/anthropics/codegraph)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Sistema GraphRAG Híbrido com NARS para Geração de Código UI.

## Diferenciais

1. **NARS para mitigar alucinações**: LLM apenas traduz, NARS raciocina com truth-values
2. **AIKR nativo**: One-shot learning sob Assumption of Insufficient Knowledge and Resources
3. **RLKGF**: Grafo como reward model (mais eficiente que RLHF)
4. **Feedback propagado**: Confiança propaga via relações SIMILAR_TO/CAN_REPLACE
5. **Hybrid Retrieval**: Fulltext + Vector + Pattern + NARS Inference

## Arquitetura

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

| Crate | Descrição |
|-------|-----------|
| `codegraph-core` | Tipos e traits compartilhados |
| `codegraph-extraction` | Pipeline de extração HTML/CSS/JS |
| `codegraph-graph` | Repositório Neo4j e relações |
| `codegraph-vector` | Qdrant embeddings + Redis cache |
| `codegraph-reasoning` | Integração NARS/OpenNARS |
| `codegraph-retrieval` | Busca híbrida (vector + graph + NARS) |
| `codegraph-feedback` | RLKGF: feedback loop com propagação |
| `codegraph-generation` | Geração de código UI |
| `codegraph-web` | Dashboard HTMX + API REST |
| `codegraph-mcp` | Servidor MCP para Claude Code |
| `codegraph-benchmark` | Suite de benchmark comparativo |
| `codegraph-cli` | Interface de linha de comando |

## Quick Start

```bash
# Iniciar infraestrutura
docker compose -f .cwa/docker/docker-compose.yml up -d

# Build
cargo build --release

# Executar API
./target/release/codegraph serve

# Ou usar Docker
docker compose up codegraph-api
```

## Caso de Uso: Criando uma Biblioteca de Componentes UI

Imagine que você é um desenvolvedor em uma empresa que tem dezenas de projetos React/Vue/Angular, cada um com componentes UI próprios. Você quer criar uma biblioteca centralizada de componentes reutilizáveis com busca inteligente.

### Passo 1: Alimentar o Knowledge Graph

Extraia componentes de seus projetos existentes:

```bash
# Via API
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

O sistema automaticamente:
- Parseia o HTML/CSS com tree-sitter
- Identifica o design system (Tailwind)
- Mapeia para a ontologia (Button → Interactive → UIElement)
- Gera embeddings vetoriais
- Cria nós e relações no Neo4j

### Passo 2: Buscar Componentes com Linguagem Natural

Quando precisar de um componente, faça uma query em português ou inglês:

```bash
curl "http://localhost:3000/api/query?q=botão+azul+com+hover+animado"

# Response:
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

O NARS reasoning analisa:
- Semântica: "azul" → `bg-blue-*` classes
- Comportamento: "hover animado" → `hover:*` + `transition`
- Confiança: 0.91 baseado em truth-values evidenciais

### Passo 3: Gerar Código Novo

Precisa de um componente que não existe? Gere com base no contexto:

```bash
curl -X POST http://localhost:3000/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Card de produto com imagem, título, preço e botão comprar",
    "design_system": "tailwind"
  }'

# Response:
{
  "html": "<article class=\"bg-white rounded-lg shadow-md overflow-hidden\">...</article>",
  "css": "/* Scoped styles */",
  "js": "// Event handlers",
  "reasoning": "Generated based on 3 similar cards in knowledge graph"
}
```

O sistema:
1. Busca componentes similares (cards existentes)
2. Usa NARS para inferir padrões comuns
3. GPT-4o gera código vanilla baseado no contexto
4. Valida HTML e injeta boas práticas

### Passo 4: Feedback Loop (RLKGF)

Se o código gerado foi útil, dê feedback:

```bash
curl -X POST http://localhost:3000/api/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "element_id": "xyz-789",
    "positive": true,
    "comment": "Perfect for the e-commerce page"
  }'
```

O feedback:
- Aumenta a confiança do componente
- Propaga para componentes SIMILAR_TO
- Melhora rankings futuros sem retreino

### Passo 5: Integração com Claude Code (MCP)

Use diretamente no Claude Code:

```bash
# No terminal, inicie o servidor MCP
codegraph mcp
```

Depois, no Claude Code:
- "Busque um componente de navbar responsiva"
- "Gere um formulário de login com validação"
- "Extraia os componentes deste arquivo HTML"

### Dashboard Web

Acesse `http://localhost:3000` para:
- Upload visual de snippets
- Query com visualização do grafo
- Métricas RLKGF em tempo real
- Histórico de gerações

## Stack

- **Backend**: Rust 1.83+ / Axum 0.8
- **Graph DB**: Neo4j 5.x
- **Vector DB**: Qdrant 1.12+
- **Cache**: Redis 7.x
- **Frontend**: HTMX 2.0 / TailwindCSS 4.0
- **Reasoning**: OpenNARS for Applications (optional, with offline fallback)

## ONA Integration

CodeGraph integrates with OpenNARS for Applications (ONA) for NARS-based reasoning.
See [docs/ona-integration.md](docs/ona-integration.md) for details.

**Running without ONA** (offline mode):
```bash
export CODEGRAPH_ONA_ENABLED=false
./target/release/codegraph serve
```

## MCP Integration

O CodeGraph expõe um servidor MCP para integração com Claude Code e outros assistentes AI.

```bash
# Adicionar ao Claude Code
codegraph mcp
```

### Tools Disponíveis

| Tool | Descrição |
|------|-----------|
| `extract_snippet` | Extrai elementos UI de HTML/CSS/JS |
| `query_ui` | Busca componentes com NARS reasoning |
| `generate_code` | Gera código UI a partir de query |
| `give_feedback` | Feedback RLKGF (thumbs up/down) |
| `get_graph_stats` | Estatísticas do knowledge graph |

### Resources

| Resource | Descrição |
|----------|-----------|
| `codegraph://metrics` | Métricas RLKGF |
| `codegraph://recent` | Gerações recentes |

## Benchmark

Compare GraphRAG+NARS vs SimpleVectorRAG:

```bash
codegraph benchmark

# Output: Markdown, JSON, HTML reports
# Métricas: Precision, Recall, F1, Hallucination Rate
# Latência: P50, P95, P99
```

## API Endpoints

| Endpoint | Método | Descrição |
|----------|--------|-----------|
| `/api/extract` | POST | Extrai elementos de código |
| `/api/query` | GET | Busca componentes |
| `/api/generate` | POST | Gera código UI |
| `/api/feedback` | POST | Registra feedback |
| `/api/stats` | GET | Estatísticas do grafo |
| `/api/metrics/rlkgf` | GET | Métricas RLKGF |
| `/health` | GET | Health check |

## Design Systems Suportados

- Material UI
- Tailwind CSS
- Chakra UI
- Bootstrap
- Custom

## Environment Variables

All configuration is done via environment variables:

### Server
| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `3000` | Server port |
| `REQUEST_TIMEOUT_SECS` | `30` | Request timeout |

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
| `REDIS_URL` | - | Redis URL (optional) |

### OpenAI
| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_API_KEY` | - | OpenAI API key |
| `OPENAI_MODEL` | `gpt-4o` | Chat model |
| `OPENAI_EMBEDDING_MODEL` | `text-embedding-3-small` | Embedding model |
| `OPENAI_MAX_TOKENS` | `4096` | Max tokens |
| `OPENAI_TEMPERATURE` | `0.7` | Temperature |

### ONA/NARS
| Variable | Default | Description |
|----------|---------|-------------|
| `CODEGRAPH_ONA_ENABLED` | `true` | Enable ONA integration |
| `ONA_HOST` | `localhost` | ONA server host |
| `ONA_PORT` | `50000` | ONA UDP port |
| `ONA_INFERENCE_CYCLES` | `100` | Inference cycles |

### Rate Limiting
| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_RPM` | `100` | Requests per minute |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Window in seconds |

### Retry & Circuit Breaker
| Variable | Default | Description |
|----------|---------|-------------|
| `RETRY_MAX_OPENAI` | `3` | Max retries for OpenAI |
| `RETRY_MAX_DB` | `2` | Max retries for databases |
| `RETRY_BASE_DELAY_MS` | `100` | Base delay between retries |
| `CIRCUIT_BREAKER_THRESHOLD` | `5` | Failures to open circuit |
| `CIRCUIT_BREAKER_TIMEOUT_SECS` | `30` | Time before half-open |

### Logging
| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_LEVEL` | `info` | Log level |
| `LOG_FORMAT` | `pretty` | Log format |

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

- **Normal Mode**: All services operational
- **Degraded Mode**: Non-critical services offline (Redis, ONA)
- **Cached Mode**: Critical services offline, serving cached responses
- **Offline Mode**: System unavailable (only when degradation disabled)

```bash
# Check system status
curl http://localhost:3000/health/ready

# Response includes degradation status
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
- **Circuit Breaker**: Opens after 5 consecutive failures, resets after 30s

### Observability

- **Request Tracing**: UUID trace_id in all requests
- **Structured Logging**: JSON logs with trace correlation
- **Prometheus Metrics**: `/metrics` endpoint for scraping
- **Health Checks**: `/health`, `/health/ready`, `/health/live`

## Testes

```bash
# Todos os testes
cargo test

# Crate específico
cargo test -p codegraph-vector
cargo test -p codegraph-benchmark
```

## License

MIT
