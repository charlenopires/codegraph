#!/bin/bash
set -e

# =============================================================================
# CodeGraph Start Script
# =============================================================================
# Starts the full CodeGraph application in Docker:
# - Neo4j (graph database)
# - Qdrant (vector database)
# - API (Rust WebSocket server on port 4040)
# - Frontend (React app on port 5100)
#
# Usage:
#   ./scripts/start.sh              # Start all services
#   ./scripts/start.sh --build      # Rebuild images before starting
#   ./scripts/start.sh --dev        # Start in development mode (local)
#   ./scripts/start.sh --down       # Stop all services
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DOCKER_DIR="$PROJECT_DIR/docker"
COMPOSE_FILE="$DOCKER_DIR/docker-compose.yml"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_banner() {
    echo ""
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}                     ${GREEN}CodeGraph${NC}                             ${BLUE}║${NC}"
    echo -e "${BLUE}║${NC}         Knowledge Graph for UI Components               ${BLUE}║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

check_requirements() {
    log_info "Checking requirements..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi

    if [ -z "$OPENAI_API_KEY" ]; then
        log_warn "OPENAI_API_KEY is not set. Some features may not work."
        log_warn "Set it with: export OPENAI_API_KEY=your-key"
    fi

    log_success "Requirements OK"
}

start_docker() {
    local build_flag=""
    if [ "$1" == "--build" ]; then
        build_flag="--build"
        log_info "Building images..."
    fi

    log_info "Starting CodeGraph services..."

    cd "$PROJECT_DIR"
    docker compose -f "$COMPOSE_FILE" up -d $build_flag

    log_info "Waiting for services to be healthy..."

    # Wait for services
    local max_attempts=60
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if docker compose -f "$COMPOSE_FILE" ps | grep -q "unhealthy\|starting"; then
            sleep 2
            attempt=$((attempt + 1))
            echo -n "."
        else
            echo ""
            break
        fi
    done

    if [ $attempt -ge $max_attempts ]; then
        log_warn "Some services may still be starting..."
    fi

    show_status
}

start_dev() {
    log_info "Starting development mode..."

    # Check if infrastructure is running
    if ! docker compose -f "$PROJECT_DIR/.cwa/docker/docker-compose.yml" ps | grep -q "Up"; then
        log_info "Starting infrastructure services..."
        docker compose -f "$PROJECT_DIR/.cwa/docker/docker-compose.yml" up -d neo4j qdrant
        sleep 5
    fi

    # Start API in background
    log_info "Starting API server on port 4040..."
    cd "$PROJECT_DIR"
    NEO4J_USER=neo4j \
    NEO4J_PASSWORD=cwa_dev_2026 \
    cargo run -p codegraph-cli -- serve --port 4040 &
    API_PID=$!

    # Wait for API to be ready
    sleep 3

    # Start frontend
    log_info "Starting frontend on port 5100..."
    cd "$PROJECT_DIR/frontend"
    bun dev &
    FRONTEND_PID=$!

    log_success "Development servers started!"
    echo ""
    echo -e "  ${GREEN}Frontend:${NC} http://localhost:5100"
    echo -e "  ${GREEN}API:${NC}      http://localhost:4040"
    echo -e "  ${GREEN}Health:${NC}   http://localhost:4040/health"
    echo ""
    echo "Press Ctrl+C to stop..."

    # Handle shutdown
    trap "kill $API_PID $FRONTEND_PID 2>/dev/null; exit 0" SIGINT SIGTERM
    wait
}

stop_docker() {
    log_info "Stopping CodeGraph services..."
    cd "$PROJECT_DIR"
    docker compose -f "$COMPOSE_FILE" down
    log_success "Services stopped"
}

show_status() {
    echo ""
    log_success "CodeGraph is running!"
    echo ""
    echo -e "  ${GREEN}Frontend:${NC}  http://localhost:5100"
    echo -e "  ${GREEN}API:${NC}       http://localhost:4040"
    echo -e "  ${GREEN}WebSocket:${NC} ws://localhost:4040/ws"
    echo -e "  ${GREEN}Health:${NC}    http://localhost:4040/health"
    echo -e "  ${GREEN}Neo4j:${NC}     http://localhost:7474"
    echo -e "  ${GREEN}Qdrant:${NC}    http://localhost:6333"
    echo ""
    echo "View logs with: docker compose -f docker/docker-compose.yml logs -f"
    echo "Stop with:      ./scripts/start.sh --down"
    echo ""
}

# =============================================================================
# Main
# =============================================================================

show_banner

case "$1" in
    --build)
        check_requirements
        start_docker --build
        ;;
    --dev)
        check_requirements
        start_dev
        ;;
    --down|--stop)
        stop_docker
        ;;
    --status)
        docker compose -f "$COMPOSE_FILE" ps
        ;;
    --logs)
        docker compose -f "$COMPOSE_FILE" logs -f
        ;;
    *)
        check_requirements
        start_docker
        ;;
esac
