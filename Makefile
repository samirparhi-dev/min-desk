# MinDesk Makefile - Production Grade Build System
# Supports both Docker and Podman container runtimes

# Variables
IMAGE_NAME ?= min-desk
IMAGE_TAG ?= latest
FULL_IMAGE = $(IMAGE_NAME):$(IMAGE_TAG)
BUILD_DATE = $(shell date +%Y%m%d)
CONFIG_FILE ?= config.json
VNC_PORT ?= 5900
WEB_PORT ?= 6080

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

# Detect container runtime
CONTAINER_RUNTIME := $(shell \
	if command -v podman >/dev/null 2>&1; then \
		echo "podman"; \
	elif command -v docker >/dev/null 2>&1; then \
		echo "docker"; \
	else \
		echo "none"; \
	fi \
)

# Check if runtime is available
ifeq ($(CONTAINER_RUNTIME),none)
$(error No container runtime found. Please install Docker or Podman)
endif

# Runtime-specific flags
ifeq ($(CONTAINER_RUNTIME),podman)
	RUNTIME_FLAGS = --userns=keep-id
	BUILD_FLAGS = --squash-all
	COMPOSE_CMD = podman-compose
else
	RUNTIME_FLAGS =
	BUILD_FLAGS = --squash
	COMPOSE_CMD = docker-compose
endif

# Default target
.PHONY: all
all: build

# Help target
.PHONY: help
help:
	@echo "$(GREEN)MinDesk - Minimal Desktop Environment$(NC)"
	@echo "$(YELLOW)Container Runtime: $(CONTAINER_RUNTIME)$(NC)"
	@echo ""
	@echo "$(BLUE)Available targets:$(NC)"
	@echo "  make build          - Build the MinDesk image"
	@echo "  make run            - Run MinDesk with VNC"
	@echo "  make run-vnc        - Run with VNC on port $(VNC_PORT)"
	@echo "  make run-web        - Run with web-based VNC access"
	@echo "  make shell          - Open shell in running container"
	@echo "  make logs           - Show container logs"
	@echo "  make stop           - Stop running containers"
	@echo "  make clean          - Remove containers and images"
	@echo "  make test           - Run tests"
	@echo "  make lint           - Run linters"
	@echo "  make export         - Export image to tar.gz"
	@echo "  make import         - Import image from tar.gz"
	@echo "  make compose-up     - Start with compose"
	@echo "  make compose-down   - Stop compose services"
	@echo "  make info           - Show runtime and image info"
	@echo ""
	@echo "$(BLUE)Configuration:$(NC)"
	@echo "  IMAGE_NAME=$(IMAGE_NAME)"
	@echo "  IMAGE_TAG=$(IMAGE_TAG)"
	@echo "  VNC_PORT=$(VNC_PORT)"
	@echo "  WEB_PORT=$(WEB_PORT)"

# Build the image
.PHONY: build
build:
	@echo "$(YELLOW)Building MinDesk with $(CONTAINER_RUNTIME)...$(NC)"
	@$(CONTAINER_RUNTIME) build \
		$(BUILD_FLAGS) \
		--tag $(FULL_IMAGE) \
		--tag $(IMAGE_NAME):$(BUILD_DATE) \
		--file Dockerfile \
		.
	@echo "$(GREEN)✓ Build successful!$(NC)"
	@$(CONTAINER_RUNTIME) images $(FULL_IMAGE) --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

# Build without cache
.PHONY: build-nocache
build-nocache:
	@echo "$(YELLOW)Building MinDesk (no cache) with $(CONTAINER_RUNTIME)...$(NC)"
	@$(CONTAINER_RUNTIME) build \
		$(BUILD_FLAGS) \
		--no-cache \
		--tag $(FULL_IMAGE) \
		--file Dockerfile \
		.
	@echo "$(GREEN)✓ Build successful!$(NC)"

# Run with VNC
.PHONY: run
run: run-vnc

.PHONY: run-vnc
run-vnc: check-image
	@echo "$(YELLOW)Starting MinDesk with VNC...$(NC)"
	@$(CONTAINER_RUNTIME) run -d \
		--name $(IMAGE_NAME)-vnc \
		--rm \
		$(RUNTIME_FLAGS) \
		-e DISPLAY=:0 \
		-e ENABLE_VNC=true \
		-p $(VNC_PORT):5900 \
		-v $(PWD)/config.json:/etc/min-desk/config.json:ro \
		-v $(PWD)/data:/home/desktop/Documents:rw \
		$(FULL_IMAGE)
	@echo "$(GREEN)✓ MinDesk started!$(NC)"
	@echo "$(BLUE)Connect via VNC: vnc://localhost:$(VNC_PORT)$(NC)"

# Run with web-based VNC
.PHONY: run-web
run-web: check-image
	@echo "$(YELLOW)Starting MinDesk with web access...$(NC)"
	@$(CONTAINER_RUNTIME) run -d \
		--name $(IMAGE_NAME)-vnc \
		--rm \
		$(RUNTIME_FLAGS) \
		-e DISPLAY=:0 \
		-e ENABLE_VNC=true \
		-p $(VNC_PORT):5900 \
		-v $(PWD)/config.json:/etc/min-desk/config.json:ro \
		$(FULL_IMAGE)
	@$(CONTAINER_RUNTIME) run -d \
		--name $(IMAGE_NAME)-novnc \
		--rm \
		-p $(WEB_PORT):8080 \
		--link $(IMAGE_NAME)-vnc:vnc \
		theasp/novnc:latest \
		--vnc vnc:5900
	@echo "$(GREEN)✓ MinDesk started with web access!$(NC)"
	@echo "$(BLUE)Open in browser: http://localhost:$(WEB_PORT)$(NC)"

# Run interactive shell
.PHONY: shell
shell: check-image
	@echo "$(YELLOW)Opening shell in MinDesk...$(NC)"
	@$(CONTAINER_RUNTIME) run -it \
		--rm \
		$(RUNTIME_FLAGS) \
		--entrypoint /bin/sh \
		$(FULL_IMAGE)

# Execute shell in running container
.PHONY: exec
exec:
	@$(CONTAINER_RUNTIME) exec -it $(IMAGE_NAME)-vnc /bin/sh

# Show logs
.PHONY: logs
logs:
	@$(CONTAINER_RUNTIME) logs -f $(IMAGE_NAME)-vnc

# Stop containers
.PHONY: stop
stop:
	@echo "$(YELLOW)Stopping MinDesk containers...$(NC)"
	@$(CONTAINER_RUNTIME) stop $(IMAGE_NAME)-vnc 2>/dev/null || true
	@$(CONTAINER_RUNTIME) stop $(IMAGE_NAME)-novnc 2>/dev/null || true
	@echo "$(GREEN)✓ Containers stopped$(NC)"

# Clean up
.PHONY: clean
clean: stop
	@echo "$(YELLOW)Cleaning up...$(NC)"
	@$(CONTAINER_RUNTIME) rm -f $(IMAGE_NAME)-vnc 2>/dev/null || true
	@$(CONTAINER_RUNTIME) rm -f $(IMAGE_NAME)-novnc 2>/dev/null || true
	@$(CONTAINER_RUNTIME) rmi -f $(FULL_IMAGE) 2>/dev/null || true
	@$(CONTAINER_RUNTIME) image prune -f
	@echo "$(GREEN)✓ Cleanup complete$(NC)"

# Deep clean (remove all related images)
.PHONY: clean-all
clean-all: clean
	@echo "$(YELLOW)Deep cleaning...$(NC)"
	@$(CONTAINER_RUNTIME) images --format "{{.Repository}}:{{.Tag}}" | \
		grep "^$(IMAGE_NAME):" | \
		xargs -r $(CONTAINER_RUNTIME) rmi -f 2>/dev/null || true
	@cargo clean 2>/dev/null || true
	@rm -rf target/ 2>/dev/null || true
	@echo "$(GREEN)✓ Deep cleanup complete$(NC)"

# Run tests
.PHONY: test
test:
	@echo "$(YELLOW)Running tests...$(NC)"
	@cargo test --all
	@echo "$(GREEN)✓ Tests passed$(NC)"

# Run linters
.PHONY: lint
lint:
	@echo "$(YELLOW)Running linters...$(NC)"
	@cargo clippy -- -D warnings
	@cargo fmt -- --check
	@echo "$(GREEN)✓ Linting passed$(NC)"

# Format code
.PHONY: fmt
fmt:
	@echo "$(YELLOW)Formatting code...$(NC)"
	@cargo fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"

# Export image
.PHONY: export
export: check-image
	@echo "$(YELLOW)Exporting image...$(NC)"
	@$(CONTAINER_RUNTIME) save $(FULL_IMAGE) | gzip > $(IMAGE_NAME)-$(IMAGE_TAG).tar.gz
	@echo "$(GREEN)✓ Image exported to $(IMAGE_NAME)-$(IMAGE_TAG).tar.gz$(NC)"
	@ls -lh $(IMAGE_NAME)-$(IMAGE_TAG).tar.gz

# Import image
.PHONY: import
import:
	@echo "$(YELLOW)Importing image...$(NC)"
	@if [ -f "$(IMAGE_NAME)-$(IMAGE_TAG).tar.gz" ]; then \
		gunzip -c $(IMAGE_NAME)-$(IMAGE_TAG).tar.gz | $(CONTAINER_RUNTIME) load; \
		echo "$(GREEN)✓ Image imported$(NC)"; \
	else \
		echo "$(RED)✗ File not found: $(IMAGE_NAME)-$(IMAGE_TAG).tar.gz$(NC)"; \
		exit 1; \
	fi

# Compose operations
.PHONY: compose-up
compose-up:
	@echo "$(YELLOW)Starting services with compose...$(NC)"
	@if command -v $(COMPOSE_CMD) >/dev/null 2>&1; then \
		$(COMPOSE_CMD) up -d; \
		echo "$(GREEN)✓ Services started$(NC)"; \
	else \
		echo "$(RED)✗ $(COMPOSE_CMD) not found$(NC)"; \
		exit 1; \
	fi

.PHONY: compose-down
compose-down:
	@echo "$(YELLOW)Stopping compose services...$(NC)"
	@if command -v $(COMPOSE_CMD) >/dev/null 2>&1; then \
		$(COMPOSE_CMD) down; \
		echo "$(GREEN)✓ Services stopped$(NC)"; \
	else \
		echo "$(RED)✗ $(COMPOSE_CMD) not found$(NC)"; \
		exit 1; \
	fi

# Show runtime and image info
.PHONY: info
info:
	@echo "$(GREEN)MinDesk Information$(NC)"
	@echo "$(BLUE)=====================================	@echo "Container Runtime: $(CONTAINER_RUNTIME)"
	@$(CONTAINER_RUNTIME) --version | head -n1
	@echo ""
	@echo "$(BLUE)Images:$(NC)"
	@$(CONTAINER_RUNTIME) images $(IMAGE_NAME) --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.Created}}"
	@echo ""
	@echo "$(BLUE)Running Containers:$(NC)"
	@$(CONTAINER_RUNTIME) ps --filter "name=$(IMAGE_NAME)" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Check if image exists
.PHONY: check-image
check-image:
	@if ! $(CONTAINER_RUNTIME) images -q $(FULL_IMAGE) >/dev/null 2>&1; then \
		echo "$(YELLOW)Image not found. Building...$(NC)"; \
		$(MAKE) build; \
	fi

# Development run with live reload
.PHONY: dev
dev:
	@echo "$(YELLOW)Starting development mode...$(NC)"
	@cargo watch -x run

# Create data directories
.PHONY: init
init:
	@echo "$(YELLOW)Initializing project directories...$(NC)"
	@mkdir -p data/documents data/backgrounds data/config
	@cp config.json data/config/config.json 2>/dev/null || true
	@echo "$(GREEN)✓ Directories created$(NC)"

# Show container statistics
.PHONY: stats
stats:
	@$(CONTAINER_RUNTIME) stats --no-stream $(IMAGE_NAME)-vnc

# Health check
.PHONY: health
health:
	@$(CONTAINER_RUNTIME) exec $(IMAGE_NAME)-vnc pgrep min-desk >/dev/null && \
		echo "$(GREEN)✓ MinDesk is healthy$(NC)" || \
		echo "$(RED)✗ MinDesk is not running$(NC)"

# Push to registry
.PHONY: push
push: check-image
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)✗ REGISTRY not set. Use: make push REGISTRY=your-registry.com$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Pushing to $(REGISTRY)/$(FULL_IMAGE)...$(NC)"
	@$(CONTAINER_RUNTIME) tag $(FULL_IMAGE) $(REGISTRY)/$(FULL_IMAGE)
	@$(CONTAINER_RUNTIME) push $(REGISTRY)/$(FULL_IMAGE)
	@echo "$(GREEN)✓ Image pushed to registry$(NC)"

# Pull from registry
.PHONY: pull
pull:
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)✗ REGISTRY not set. Use: make pull REGISTRY=your-registry.com$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Pulling from $(REGISTRY)/$(FULL_IMAGE)...$(NC)"
	@$(CONTAINER_RUNTIME) pull $(REGISTRY)/$(FULL_IMAGE)
	@$(CONTAINER_RUNTIME) tag $(REGISTRY)/$(FULL_IMAGE) $(FULL_IMAGE)
	@echo "$(GREEN)✓ Image pulled from registry$(NC)"

# Version information
.PHONY: version
version:
	@echo "MinDesk Version: $(IMAGE_TAG)"
	@echo "Build Date: $(BUILD_DATE)"
	@echo "Runtime: $(CONTAINER_RUNTIME)"

.DEFAULT_GOAL := help
