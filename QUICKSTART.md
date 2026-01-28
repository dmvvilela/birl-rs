# Sandwich Rust - Quick Start Guide

## 🚀 Get Started in 30 Seconds

### 1. Set up environment

```bash
cd /Users/danvilela/Code/Work/DivBrands/sandwich-rs

# Copy and edit environment file
cp .env.example .env
# Edit .env with your AWS credentials
```

### 2. Try the CLI with examples

```bash
# List all available examples
cargo run --bin sandwich-cli -- examples

# Try the basic example (single hoodie)
cargo run --bin sandwich-cli -- compose --example basic -o result.jpg

# Full outfit example
cargo run --bin sandwich-cli -- compose --example full-outfit -o outfit.jpg

# With patches
cargo run --bin sandwich-cli -- compose --example with-patches -o patches.jpg
```

### 3. Custom compositions

```bash
# Front view with custom items
cargo run --bin sandwich-cli -- compose \
  --params "hoodies/baerskin4-black,pants/cargo-darkgreen,hats/beanie-black" \
  -o custom.jpg

# Back view
cargo run --bin sandwich-cli -- compose \
  --example full-outfit \
  --view back \
  -o back-view.jpg

# With verbose logging
cargo run --bin sandwich-cli -- -v compose --example basic -o test.jpg

# Bypass cache to regenerate
cargo run --bin sandwich-cli -- compose \
  --example basic \
  --bypass-cache \
  -o fresh.jpg
```

## 📊 CLI Commands Reference

### Compose Command
```bash
cargo run --bin sandwich-cli -- compose [OPTIONS]

Options:
  --view <VIEW>              View to render [default: front]
                             Values: front, back, side, left, right
  -p, --params <PARAMS>      Parameters: "category/sku,category/sku,..."
  -e, --example <EXAMPLE>    Use a pre-made example
  -o, --output <OUTPUT>      Output file path
  -b, --bypass-cache         Bypass cache and force regeneration
  -v, --verbose              Enable verbose logging
  -h, --help                 Print help
```

### Available Examples
- `basic` - Single black hoodie
- `full-outfit` - Hoodie, pants, and beanie
- `with-patches` - Hoodie with American flag patch
- `jacket-outfit` - Jacket over hoodie with pants
- `gloves-hat` - Full winter outfit with gloves and hat
- `outer-jacket` - Greenland jacket over hoodie

### Other Commands
```bash
# List all examples with descriptions
cargo run --bin sandwich-cli -- examples

# Show cache statistics
cargo run --bin sandwich-cli -- stats

# Get help
cargo run --bin sandwich-cli -- --help
cargo run --bin sandwich-cli -- compose --help
```

## 🌐 Start the Web Server

```bash
# Development mode
cargo run --bin sandwich-server

# Production mode (optimized)
cargo run --release --bin sandwich-server

# The server will start on http://localhost:3000
```

### Test the API

```bash
# Health check
curl http://localhost:3000/health

# Create composite (basic)
curl -X POST http://localhost:3000/create \
  -H "Content-Type: application/json" \
  -d '{"p": "hoodies/baerskin4-black"}' \
  --output result.jpg

# Create composite (full outfit, back view)
curl -X POST http://localhost:3000/create \
  -H "Content-Type: application/json" \
  -d '{
    "p": "hoodies/baerskin4-black,pants/cargo-darkgreen,hats/beanie-black",
    "view": "back"
  }' \
  --output back-view.jpg

# Bypass cache
curl -X POST http://localhost:3000/create \
  -H "Content-Type: application/json" \
  -d '{
    "p": "hoodies/baerskin4-black",
    "bypassCache": true
  }' \
  --output fresh.jpg

# Get products
curl http://localhost:3000/products
```

## 🐳 Docker Quick Start

```bash
# Build Docker image
docker build -t sandwich-server .

# Run with docker-compose (easiest)
docker-compose up

# Or run manually
docker run -p 3000:3000 \
  -e AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
  -e AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY \
  -e AWS_REGION=$AWS_REGION \
  -e AWS_BUCKET_NAME=$AWS_BUCKET_NAME \
  sandwich-server
```

## 🔧 Development Commands

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Build optimized release
cargo build --release --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace

# Using Makefile
make test          # Run tests
make build         # Build release
make run-server    # Start server
make run-cli       # Run CLI
make help          # Show all commands
```

## 📝 Parameter Format

Parameters follow this format: `category/sku,category/sku,...`

### Valid Categories
- `hoodies` - Hoodies
- `pants` - Pants
- `jackets` - Jackets (auto-categorized to outer-jackets if greenland)
- `gloves` - Gloves (auto-categorized to top/bottom based on type)
- `hats` - Hats
- `tops` - Tops
- `patches-left` - Left patches
- `patches-right` - Right patches

### Examples
```bash
# Single item
"hoodies/baerskin4-black"

# Multiple items
"hoodies/baerskin4-black,pants/cargo-darkgreen"

# With patches
"hoodies/baerskin4-black,patches-left/americanflagpatch-red"

# Full outfit
"hoodies/baerskin4-black,pants/cargo-black,hats/beanie-black,gloves/baerskinleatherlinedgloves-black"
```

## ⚙️ Environment Variables

Required:
```bash
AWS_ACCESS_KEY_ID=your_key
AWS_SECRET_ACCESS_KEY=your_secret
AWS_REGION=us-east-1
AWS_BUCKET_NAME=your-bucket-name
```

Optional:
```bash
PORT=3000                    # Server port (default: 3000)
RUST_LOG=info               # Log level (trace, debug, info, warn, error)
```

## 🐛 Troubleshooting

### "Base plate not found"
- Check your AWS credentials are set correctly
- Verify the bucket name in `.env`
- Ensure images exist at `sandwich/{view}/plate/` in your bucket

### "Warning: AWS_BUCKET_NAME not set"
- Copy `.env.example` to `.env` and set your bucket name

### CLI option conflict errors
- Use `--view` (not `-v`) for view selection
- Use `-v` for verbose logging
- Example: `cargo run --bin sandwich-cli -- -v compose --view back --example basic`

### Build errors
```bash
# Clean and rebuild
cargo clean
cargo build --workspace
```

## 📚 More Information

- **Full Documentation**: See [README.md](README.md)
- **API Details**: See README.md "Web Server" section
- **Implementation Details**: See [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- **Version History**: See [CHANGELOG.md](CHANGELOG.md)

## 🎯 Common Use Cases

### Testing a specific combination
```bash
cargo run --bin sandwich-cli -- compose \
  --params "hoodies/baerskin4-black,jackets/softshell-grey" \
  -o test.jpg
```

### Generating all views of an outfit
```bash
for view in front back side left right; do
  cargo run --bin sandwich-cli -- compose \
    --example full-outfit \
    --view $view \
    -o "outfit-${view}.jpg"
done
```

### Performance testing
```bash
# With timing
time cargo run --release --bin sandwich-cli -- compose \
  --example full-outfit \
  -o perf-test.jpg

# With verbose logging to see cache hits
cargo run --bin sandwich-cli -- -v compose \
  --example full-outfit \
  -o test.jpg
```

### API load testing
```bash
# Install Apache Bench
brew install ab  # macOS

# Run load test (100 requests, 10 concurrent)
ab -n 100 -c 10 -p request.json -T application/json \
  http://localhost:3000/create

# request.json contents:
# {"p":"hoodies/baerskin4-black","view":"front"}
```

---

**Need help?** Check the full [README.md](README.md) or [file an issue](https://github.com/divbrands/sandwich-rs/issues).
