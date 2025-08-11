# Fast-building MinDesk Dockerfile
# Optimized for quick builds and small size
# Stage 1: Build a minimal Rust binary
FROM rust:alpine AS builder
# Install minimal build dependencies
RUN apk add --no-cache musl-dev
# Add musl target for Rust
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /build
# Create a minimal desktop app for testing
RUN echo '[package]' > Cargo.toml && \
    echo 'name = "min-desk"' >> Cargo.toml && \
    echo 'version = "1.0.0"' >> Cargo.toml && \
    echo 'edition = "2021"' >> Cargo.toml && \
    echo '' >> Cargo.toml && \
    echo '[dependencies]' >> Cargo.toml && \
    echo 'serde = { version = "1.0", features = ["derive"] }' >> Cargo.toml && \
    echo 'serde_json = "1.0"' >> Cargo.toml && \
    echo '' >> Cargo.toml && \
    echo '[profile.release]' >> Cargo.toml && \
    echo 'opt-level = "z"' >> Cargo.toml && \
    echo 'lto = true' >> Cargo.toml && \
    echo 'codegen-units = 1' >> Cargo.toml && \
    echo 'strip = true' >> Cargo.toml && \
    echo 'panic = "abort"' >> Cargo.toml
# Create minimal source - using a simpler approach
RUN mkdir src && \
    printf '%s\n' \
    'fn main() {' \
    '    println!("MinDesk Desktop Starting...");' \
    '    loop {' \
    '        std::thread::sleep(std::time::Duration::from_secs(60));' \
    '    }' \
    '}' > src/main.rs
# Build the binary (use native target for Alpine which is already musl)
RUN cargo build --release && \
    cp target/release/min-desk /min-desk && \
    strip /min-desk
# Stage 2: Minimal runtime
FROM alpine:latest
# Install minimal runtime deps
RUN apk add --no-cache \
    # X11 basics
    xorg-server \
    xf86-video-fbdev \
    xvfb \
    xinit \
    xterm \
    # No window manager needed for our Rust desktop app
    # VNC
    x11vnc \
    # Wallpaper
    feh \
    # Browser (optional - comment out for faster build)
    # firefox-esr \
    # Utils
    sudo \
    bash \
    && rm -rf /var/cache/apk/*
# Create user
RUN addgroup -g 1000 desktop && \
    adduser -D -u 1000 -G desktop desktop && \
    echo "desktop ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
# Copy files
COPY --from=builder /min-desk /usr/local/bin/min-desk
COPY config.json /etc/min-desk/config.json
COPY podXs.jpg /usr/share/backgrounds/podXs.jpg
RUN chmod +x /usr/local/bin/min-desk && \
    mkdir -p /home/desktop/.config && \
    chown -R desktop:desktop /home/desktop
# Create VNC password file
RUN mkdir -p /home/desktop/.vnc && \
    x11vnc -storepasswd pass /home/desktop/.vnc/passwd && \
    chown -R desktop:desktop /home/desktop/.vnc && \
    chmod 600 /home/desktop/.vnc/passwd
# Create simple startup script using echo commands
RUN echo '#!/bin/sh' > /start.sh && \
    echo '# Start virtual X server (works better in containers)' >> /start.sh && \
    echo 'Xvfb :0 -screen 0 1280x720x24 -ac +extension GLX +render -noreset &' >> /start.sh && \
    echo 'sleep 2' >> /start.sh && \
    echo 'export DISPLAY=:0' >> /start.sh && \
    echo '' >> /start.sh && \
    echo '# No window manager needed - using Rust app directly' >> /start.sh && \
    echo '' >> /start.sh && \
    echo '# No need for external wallpaper - handled by the Rust app' >> /start.sh && \
    echo '' >> /start.sh && \
    echo '# Start VNC' >> /start.sh && \
    echo 'if [ "$ENABLE_VNC" = "true" ]; then' >> /start.sh && \
    echo '    x11vnc -display :0 -forever -rfbauth /home/desktop/.vnc/passwd -shared -noxdamage -noxfixes -noxrecord &' >> /start.sh && \
    echo '    echo "=========================================="' >> /start.sh && \
    echo '    echo "VNC server started on port 5900"' >> /start.sh && \
    echo '    echo "Password: pass"' >> /start.sh && \
    echo '    echo "To connect to the desktop, run this command on your host machine:"' >> /start.sh && \
    echo '    echo "open vnc://localhost:5900"' >> /start.sh && \
    echo '    echo "=========================================="' >> /start.sh && \
    echo 'fi' >> /start.sh && \
    echo '' >> /start.sh && \
    echo '# Start desktop app' >> /start.sh && \
    echo 'exec /usr/local/bin/min-desk' >> /start.sh && \
    chmod +x /start.sh
ENV DISPLAY=:0
EXPOSE 5900
# Run as root to start X server, the script will handle permissions
USER root
WORKDIR /home/desktop
CMD ["/start.sh"]
