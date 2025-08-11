#!/bin/sh
# Build script for MinDesk - Minimal Desktop Environment
# Compatible with both Docker and Podman

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="${IMAGE_NAME:-min-desk}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
CONTAINER_RUNTIME=""

# Detect container runtime
detect_runtime() {
    if command -v podman >/dev/null 2>&1; then
        CONTAINER_RUNTIME="podman"
        echo -e "${GREEN}✓${NC} Using Podman"
    elif command -v docker >/dev/null 2>&1; then
        CONTAINER_RUNTIME="docker"
        echo -e "${GREEN}✓${NC} Using Docker"
    else
        echo -e "${RED}✗${NC} Neither Docker nor Podman found. Please install one."
        exit 1
    fi
}

# Build the image
build_image() {
    echo -e "${YELLOW}Building MinDesk image...${NC}"

    # Build with optimizations for size
    $CONTAINER_RUNTIME build \
        --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
        --tag "${IMAGE_NAME}:$(date +%Y%m%d)" \
        --file Dockerfile \
        --squash \
        --no-cache \
        .

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} Build successful!"

        # Show image size
        echo -e "\n${YELLOW}Image information:${NC}"
        $CONTAINER_RUNTIME images "${IMAGE_NAME}:${IMAGE_TAG}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
    else
        echo -e "${RED}✗${NC} Build failed!"
        exit 1
    fi
}

# Clean up old images
cleanup() {
    echo -e "\n${YELLOW}Cleaning up old images...${NC}"

    # Remove dangling images
    $CONTAINER_RUNTIME image prune -f

    # Remove old versions (keep last 2)
    if [ "$CONTAINER_RUNTIME" = "docker" ]; then
        docker images "${IMAGE_NAME}" --format "{{.Tag}}" | \
            grep -E '^[0-9]{8}$' | \
            sort -r | \
            tail -n +3 | \
            xargs -r -I {} docker rmi "${IMAGE_NAME}:{}" 2>/dev/null || true
    fi

    echo -e "${GREEN}✓${NC} Cleanup complete"
}

# Export image for distribution
export_image() {
    echo -e "\n${YELLOW}Exporting image...${NC}"

    OUTPUT_FILE="${IMAGE_NAME}-${IMAGE_TAG}.tar.gz"

    $CONTAINER_RUNTIME save "${IMAGE_NAME}:${IMAGE_TAG}" | gzip > "$OUTPUT_FILE"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} Image exported to: $OUTPUT_FILE"
        echo -e "    Size: $(du -h "$OUTPUT_FILE" | cut -f1)"
    else
        echo -e "${RED}✗${NC} Export failed!"
        exit 1
    fi
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -h, --help          Show this help message
    -c, --clean         Clean up old images after build
    -e, --export        Export image to tar.gz file
    -t, --tag TAG       Set image tag (default: latest)
    -n, --name NAME     Set image name (default: min-desk)
    --no-cache          Build without cache

Examples:
    $0                  # Basic build
    $0 -c               # Build and cleanup
    $0 -e               # Build and export
    $0 -t v1.0.0        # Build with custom tag

EOF
}

# Main script
main() {
    echo -e "${GREEN}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}   MinDesk - Minimal Desktop Builder   ${NC}"
    echo -e "${GREEN}═══════════════════════════════════════${NC}\n"

    # Parse arguments
    CLEAN_AFTER=false
    EXPORT_AFTER=false
    NO_CACHE=""

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            -c|--clean)
                CLEAN_AFTER=true
                shift
                ;;
            -e|--export)
                EXPORT_AFTER=true
                shift
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -n|--name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            --no-cache)
                NO_CACHE="--no-cache"
                shift
                ;;
            *)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Execute build steps
    detect_runtime
    build_image

    if [ "$CLEAN_AFTER" = true ]; then
        cleanup
    fi

    if [ "$EXPORT_AFTER" = true ]; then
        export_image
    fi

    echo -e "\n${GREEN}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ MinDesk build completed successfully!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════${NC}"

    # Show run instructions
    echo -e "\n${YELLOW}To run the desktop:${NC}"
    echo -e "  $CONTAINER_RUNTIME run -it --rm \\"
    echo -e "    -e DISPLAY=:0 \\"
    echo -e "    -e ENABLE_VNC=true \\"
    echo -e "    -p 5900:5900 \\"
    echo -e "    -v /tmp/.X11-unix:/tmp/.X11-unix:rw \\"
    echo -e "    ${IMAGE_NAME}:${IMAGE_TAG}"
    echo -e "\n${YELLOW}Connect via VNC:${NC}"
    echo -e "  vnc://localhost:5900"
}

# Run main function
main "$@"
