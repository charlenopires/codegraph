# OpenNARS for Applications (ONA) Integration

CodeGraph uses ONA for NARS-based reasoning to improve query understanding and reduce hallucinations in code generation.

## Architecture

```
┌──────────────────┐      ┌──────────────────┐
│  ReasoningPipeline│──UDP──│  ONA Container   │
│  (Rust)          │ 50000 │  (Docker)        │
└──────────────────┘      └──────────────────┘
```

## Running with ONA

### Docker Compose (Recommended)

```bash
docker compose -f .cwa/docker/docker-compose.yml up -d
```

This starts the ONA container along with other services.

### Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `ONA_HOST` | `localhost` | ONA server hostname |
| `ONA_PORT` | `50000` | ONA UDP port |
| `ONA_TIMESTEP` | `10000000` | Nanoseconds per inference cycle |
| `CODEGRAPH_ONA_ENABLED` | `true` | Enable/disable ONA integration |

## Fallback Mode (Offline)

When ONA is unavailable or disabled, CodeGraph operates in **offline mode**:

### Automatic Fallback

The system automatically falls back to offline mode when:
1. ONA container is not running
2. Connection to ONA fails
3. ONA integration is explicitly disabled

### Disabling ONA

To run without ONA (offline mode only):

```bash
export CODEGRAPH_ONA_ENABLED=false
./target/release/codegraph serve
```

Or in docker-compose:

```yaml
services:
  codegraph-api:
    environment:
      - CODEGRAPH_ONA_ENABLED=false
```

### Offline Mode Behavior

In offline mode:
- Query translation uses rule-based patterns only
- No NARS inference is performed
- `derived_statements` in results will be empty
- Basic search terms are still extracted
- System logs a warning: `"ONA integration disabled, using offline mode"`

### Feature Comparison

| Feature | With ONA | Offline Mode |
|---------|----------|--------------|
| Query translation | Rule-based + NARS | Rule-based only |
| Inference | 100 cycles | None |
| Derived knowledge | Yes | No |
| Latency | ~200ms | ~10ms |
| Hallucination mitigation | Full | Limited |

## Health Check

The ONA container includes a health check that:
1. Verifies the NAR binary is executable
2. Sends a test Narsese statement via UDP
3. Reports healthy if UDP communication succeeds

Check ONA health:

```bash
docker inspect --format='{{.State.Health.Status}}' cwa-ona
```

## Troubleshooting

### ONA not starting

Check container logs:
```bash
docker logs cwa-ona
```

### Connection refused

1. Verify ONA is running: `docker ps | grep ona`
2. Check port is exposed: `docker port cwa-ona`
3. Test UDP connectivity: `nc -u localhost 50000`

### High latency

Reduce inference cycles:
```bash
export ONA_TIMESTEP=5000000  # Faster cycles
```

Or reduce cycles in code:
```rust
let pipeline = ReasoningPipeline::new()
    .with_inference_cycles(50);  // Default is 100
```

## Building ONA Manually

If you need to build ONA outside Docker:

```bash
git clone https://github.com/opennars/OpenNARS-for-Applications.git
cd OpenNARS-for-Applications
./build.sh
./NAR UDPNAR 0.0.0.0 50000 10000000 false
```

## References

- [OpenNARS for Applications](https://github.com/opennars/OpenNARS-for-Applications)
- [NARS Theory](https://github.com/opennars/opennars/wiki)
