#!/bin/bash
set -e

# =============================================================================
# CodeGraph Development Script
# =============================================================================
# Starts the application in development mode (no Docker for app, only infra)
#
# Ports:
#   - API:      4040 (Rust WebSocket)
#   - Frontend: 5100 (Vite dev server)
#   - Neo4j:    7474, 7687
#   - Qdrant:   6333, 6334
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}              ${GREEN}CodeGraph Development Mode${NC}                  ${BLUE}║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if infrastructure is running
if ! docker ps | grep -q "cwa-neo4j"; then
    echo -e "${YELLOW}[INFO]${NC} Starting infrastructure services..."
    docker compose -f "$PROJECT_DIR/.cwa/docker/docker-compose.yml" up -d neo4j qdrant
    echo -e "${YELLOW}[INFO]${NC} Waiting for services to be ready..."
    sleep 8
fi

# Kill any existing processes on our ports
pkill -f "codegraph serve" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
sleep 1

# Start API
echo -e "${BLUE}[INFO]${NC} Starting API on port 4040..."
cd "$PROJECT_DIR"
NEO4J_USER=neo4j \
NEO4J_PASSWORD=cwa_dev_2026 \
QDRANT_URL=http://localhost:6334 \
cargo run -p codegraph-cli --release -- serve --port 4040 &
API_PID=$!

# Wait for API to be ready
echo -e "${BLUE}[INFO]${NC} Waiting for API to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:4040/health > /dev/null 2>&1; then
        break
    fi
    sleep 1
done

# Start frontend
echo -e "${BLUE}[INFO]${NC} Starting frontend on port 5100..."
cd "$PROJECT_DIR/frontend"
bun dev &
FRONTEND_PID=$!

sleep 3

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}                   ${BLUE}CodeGraph Running${NC}                       ${GREEN}║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${GREEN}Frontend:${NC}  http://localhost:5100"
echo -e "  ${GREEN}API:${NC}       http://localhost:4040"
echo -e "  ${GREEN}WebSocket:${NC} ws://localhost:4040/ws"
echo -e "  ${GREEN}Health:${NC}    http://localhost:4040/health"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Handle shutdown
cleanup() {
    echo ""
    echo -e "${BLUE}[INFO]${NC} Stopping services..."
    kill $API_PID $FRONTEND_PID 2>/dev/null || true
    echo -e "${GREEN}[OK]${NC} Services stopped"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for processes
wait
