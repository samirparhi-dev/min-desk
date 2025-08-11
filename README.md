# MinDesk - Minimal Desktop Environment

A lightweight, modern desktop environment built with Rust and Iced framework, optimized for Alpine Linux and Talos OS.

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Size](https://img.shields.io/badge/image%20size-<150MB-orange)
![Rust](https://img.shields.io/badge/rust-1.75+-red)

## 🎯 Features

### Core Applications
- **📁 File Manager** - Navigate, create, and manage files and folders
- **📦 Package Manager** - Install and manage Alpine packages (apk)
- **🌐 Web Browser** - Lightweight browser with minimal mode for text-based browsing

### Design Principles
- **Minimal** - Only essential features, no bloat
- **Modern** - Clean, contemporary UI with emoji icons
- **Fast** - Built with Rust for optimal performance
- **Small** - Docker image under 150MB
- **Configurable** - JSON-based configuration system
- **Container-First** - Designed for containerized environments

## 🚀 Quick Start

### Using Docker

```bash
# Build the image
docker build -t min-desk:latest .

# Run with VNC support
docker run -it --rm \
  -e DISPLAY=:0 \
  -e ENABLE_VNC=true \
  -p 5900:5900 \
  min-desk:latest

# Connect via VNC
# URL: vnc://localhost:5900
# Or run the helper script to open VNC automatically
# ./open-vnc-client.sh
```

### Using Podman

```bash
# Build the image
podman build -t min-desk:latest .

# Run with VNC support
podman run -it --rm \
  -e DISPLAY=:0 \
  -e ENABLE_VNC=true \
  -p 5900:5900 \
  min-desk:latest
```

### Using Docker Compose

```bash
# Start the desktop with NoVNC web access
docker-compose up -d

# Access via VNC: vnc://localhost:5900 (use ./open-vnc-client.sh to open automatically)
# Access via Web: http://localhost:6080
```

## 📋 System Requirements

### Minimum
- **CPU**: 1 core
- **RAM**: 512MB
- **Storage**: 200MB
- **Display**: 800x600

### Recommended
- **CPU**: 2+ cores
- **RAM**: 1GB+
- **Storage**: 500MB
- **Display**: 1280x720 or higher

### Compatible OS
- Alpine Linux 3.18+
- Talos OS 1.5+
- Any Linux with container runtime

## 🔧 Configuration

### Configuration File (`config.json`)

```json
{
  "desktop": {
    "wallpaper": "/usr/share/backgrounds/default.png",
    "font_name": "Inter",
    "font_size": 12,
    "theme": "dark"
  },
  "applications": {
    "file_manager": {
      "enabled": true,
      "icon": "📁",
      "default_path": "/home"
    },
    "package_manager": {
      "enabled": true,
      "icon": "📦",
      "backend": "apk"
    },
    "browser": {
      "enabled": true,
      "icon": "🌐",
      "homepage": "https://start.duckduckgo.com",
      "minimal_mode": true
    }
  },
  "packages_to_install": [
    "firefox-esr",
    "ttf-liberation"
  ],
  "system": {
    "dpi": 96,
    "vsync": true,
    "compositor": false
  }
}
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DISPLAY` | X11 display | `:0` |
| `ENABLE_VNC` | Enable VNC server | `false` |
| `AUTO_OPEN_VNC` | Provide instructions to open VNC client automatically | `false` |
| `XDG_RUNTIME_DIR` | Runtime directory | `/tmp/runtime-desktop` |

## 🏗️ Building from Source

### Prerequisites
- Rust 1.75+
- Alpine Linux build environment
- Docker or Podman

### Build Steps

```bash
# Clone the repository
git clone https://github.com/yourusername/min-desk.git
cd min-desk

# Build with the provided script
./build.sh

# Or build manually
cargo build --release
```

### Build Options

```bash
# Build with cleanup
./build.sh -c

# Build and export image
./build.sh -e

# Build with custom tag
./build.sh -t v1.0.0
```

## 📁 Project Structure

```
min-desk/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── file_manager.rs   # File manager module
│   ├── package_manager.rs # Package manager module
│   └── browser.rs        # Web browser module
├── Cargo.toml            # Rust dependencies
├── Dockerfile            # Container definition
├── config.json           # Default configuration
├── docker-compose.yml    # Compose configuration
└── build.sh             # Build script
```

## 🎨 Customization

### Adding Custom Wallpaper

1. Mount your wallpaper directory:
```bash
docker run -v /path/to/wallpapers:/usr/share/backgrounds ...
```

2. Update config.json:
```json
{
  "desktop": {
    "wallpaper": "/usr/share/backgrounds/custom.png"
  }
}
```

### Installing Additional Packages

Add packages to `config.json`:
```json
{
  "packages_to_install": [
    "firefox-esr",
    "git",
    "vim",
    "your-package"
  ]
}
```

### Custom Fonts

1. Mount font directory:
```bash
docker run -v /path/to/fonts:/usr/share/fonts/custom ...
```

2. Update configuration:
```json
{
  "desktop": {
    "font_name": "YourCustomFont"
  }
}
```

## 🐳 Deployment

### Kubernetes/Talos OS

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: min-desk
spec:
  replicas: 1
  selector:
    matchLabels:
      app: min-desk
  template:
    metadata:
      labels:
        app: min-desk
    spec:
      containers:
      - name: min-desk
        image: min-desk:latest
        ports:
        - containerPort: 5900
        env:
        - name: ENABLE_VNC
          value: "true"
```

### Systemd Service

```ini
[Unit]
Description=MinDesk Desktop Environment
After=docker.service

[Service]
Type=simple
ExecStart=/usr/bin/docker run --rm \
  --name min-desk \
  -p 5900:5900 \
  -e ENABLE_VNC=true \
  min-desk:latest
ExecStop=/usr/bin/docker stop min-desk
Restart=always

[Install]
WantedBy=multi-user.target
```

## 🔍 Troubleshooting

### VNC Connection Issues
- Ensure port 5900 is accessible
- Check firewall settings
- Verify VNC is enabled: `ENABLE_VNC=true`
- Use the helper script: `./open-vnc-client.sh` to open VNC client automatically
- On macOS, manually open with: `open vnc://localhost:5900`

### Black Screen
- Check X server is running: `ps aux | grep X`
- Verify graphics drivers are loaded
- Try software rendering: remove `/dev/dri` mount

### Package Installation Fails
- Ensure container has internet access
- Check Alpine repositories are accessible
- Verify sudo permissions are configured

### Performance Issues
- Allocate more memory to container
- Enable hardware acceleration (mount `/dev/dri`)
- Reduce resolution in VNC client

## 🛠️ Development

### Running Tests
```bash
cargo test
```

### Debug Mode
```bash
RUST_LOG=debug cargo run
```

### Contributing
1. Fork the repository
2. Create a feature branch
3. Commit changes
4. Push to branch
5. Open a Pull Request

## 📊 Performance

| Metric | Value |
|--------|-------|
| Startup Time | < 5 seconds |
| Memory Usage | ~100MB idle |
| CPU Usage | < 5% idle |
| Image Size | < 150MB |

## 🔒 Security

- Runs as non-root user
- Minimal attack surface
- No unnecessary services
- Regular security updates
- Sandboxed applications

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details

## 🙏 Acknowledgments

- [Iced](https://github.com/iced-rs/iced) - GUI framework
- [Alpine Linux](https://alpinelinux.org/) - Base OS
- [Rust](https://www.rust-lang.org/) - Programming language

## 📞 Support

- Issues: [GitHub Issues](https://github.com/yourusername/min-desk/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/min-desk/discussions)
- Email: support@mindesk.example.com

---

**MinDesk** - Minimal Desktop, Maximum Efficiency 🚀