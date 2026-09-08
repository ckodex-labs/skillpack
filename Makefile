# SkillPack Makefile
# Build, test, and publish SkillPack components

.PHONY: all build test check release publish clean ci

# Configuration
REGISTRY ?= ghcr.io
NAMESPACE ?= ckodex
VERSION ?= $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "skillpack-domain") | .version')

# Binaries
SKILLPACK_CLI := target/release/skillpack
SKILLPACK_SERVER := target/release/skillpack-server

all: build test

# Build all binaries
build:
	cargo build --release

# Build CLI only
build-cli:
	cargo build --release --bin skillpack

# Build server only
build-server:
	cargo build --release --bin skillpack-server

# Run all tests
test:
	cargo test --workspace

# Run clippy and format checks
check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

# CI-parity release gate (fmt, clippy strict, tests, smoke) via xtask
ci:
	cargo run --quiet -p xtask -- ci

# Run skill assessment on current directory
assess:
	cargo run --bin skillpack -- check .

# Generate SARIF report
report-sarif:
	cargo run --bin skillpack -- report . --format sarif --output skillpack-results.sarif

# Build release artifacts
release: build
	@mkdir -p dist
	cp $(SKILLPACK_CLI) dist/skillpack-$(shell uname -m)-$(shell uname -s | tr '[:upper:]' '[:lower:]')
	cp $(SKILLPACK_SERVER) dist/skillpack-server-$(shell uname -m)-$(shell uname -s | tr '[:upper:]' '[:lower:]')
	cd dist && shasum -a 256 * > checksums.txt

# Publish skill bundle to OCI registry
publish: build
	@echo "Publishing SkillPack v$(VERSION) to $(REGISTRY)/$(NAMESPACE)"
	@if [ -f "skill-bundle.cnsb.json" ]; then \
		echo "Pushing skill bundle..."; \
		$(SKILLPACK_CLI) oci push \
			--registry $(REGISTRY)/$(NAMESPACE)/skillpack \
			--tag $(VERSION) \
			--file skill-bundle.cnsb.json \
			--media-type application/vnd.ckodex.cnsb.v1+json; \
	else \
		echo "No skill-bundle.cnsb.json found, creating from current skill..."; \
		$(SKILLPACK_CLI) bundle create . --output skill-bundle.cnsb.json; \
		$(SKILLPACK_CLI) oci push \
			--registry $(REGISTRY)/$(NAMESPACE)/skillpack \
			--tag $(VERSION) \
			--file skill-bundle.cnsb.json \
			--media-type application/vnd.ckodex.cnsb.v1+json; \
	fi

# Sign artifact with cosign
sign:
	@echo "Signing artifact with Sigstore/cosign..."
	cosign sign --yes $(REGISTRY)/$(NAMESPACE)/skillpack:$(VERSION)

# Verify signature
verify:
	@echo "Verifying signature..."
	cosign verify $(REGISTRY)/$(NAMESPACE)/skillpack:$(VERSION)

# Generate SBOM
sbom:
	@echo "Generating SBOM..."
	cargo sbom > sbom.cyclonedx.json

# Generate provenance attestation
provenance:
	@echo "Generating SLSA provenance..."
	slsa-provenance generate \
		--tag v$(VERSION) \
		--output provenance.json

# Full supply chain verification
verify-supply-chain: verify sbom
	@echo "Supply chain verification complete"
	@echo "  ✓ Signature verified"
	@echo "  ✓ SBOM generated"

# Install CLI locally
install: build-cli
	cp $(SKILLPACK_CLI) /usr/local/bin/skillpack
	@echo "Installed skillpack to /usr/local/bin/skillpack"

# Clean build artifacts
clean:
	cargo clean
	rm -rf dist
	rm -f skillpack-results.sarif sbom.cyclonedx.json provenance.json

# Canonical store operations (via skillpack CLI)
store-status:
	cargo run --bin skillpack -- store status

store-sync:
	cargo run --bin skillpack -- store sync

store-migrate:
	cargo run --bin skillpack -- store migrate

store-boundary:
	cargo run --bin skillpack -- store check-boundary

# Development server
dev-server:
	cargo run --bin skillpack-server

# Watch mode for development
watch:
	cargo watch -x 'run --bin skillpack -- check .'

# Documentation
docs:
	cargo doc --no-deps --workspace
	@echo "Documentation generated at target/doc"

# ========================================
# Unified Client Type Generation
# ========================================

SCHEMA := schemas/skills-specs-next/client-model.schema.json
GEN_TIMESTAMP := $(shell date -u +%Y-%m-%dT%H:%M:%SZ)

.PHONY: generate-types check-schema-sync

generate-types:
	@echo "Generating canonical TypeScript types (once)..."
	@tmp_ts=$$(mktemp); \
	npx --prefix dashboard json2ts $(SCHEMA) --output "$$tmp_ts" --unreachableDefinitions >/dev/null 2>&1; \
	{ \
	  echo "/* Auto-generated from $(SCHEMA)"; \
	  echo " * Generated at: $(GEN_TIMESTAMP)"; \
	  echo " * Do not edit manually. Run \`make generate-types\` to regenerate."; \
	  echo " */"; \
	  cat "$$tmp_ts"; \
	} > dashboard/src/generated/client-model.ts; \
	cp dashboard/src/generated/client-model.ts vscode-extension/src/generated/client-model.ts; \
	rm -f "$$tmp_ts"
	@echo "TypeScript types written to dashboard/ and vscode-extension/"
	@echo "Generating Rust types..."
	python3 scripts/generate-rust-types.py $(SCHEMA) crates/skillpack-adapters/src/generated/client_model.rs $(GEN_TIMESTAMP)
	@echo "Generating Swift types..."
	cd macos/skills-ecosystem && $(MAKE) generate-types GEN_TIMESTAMP=$(GEN_TIMESTAMP)
	@echo "All types generated."

check-schema-sync:
	@echo "Checking for generated file drift..."
	@tmp_dash=$$(mktemp); tmp_dash_committed=$$(mktemp); \
	npx --prefix dashboard json2ts $(SCHEMA) --output "$$tmp_dash" --unreachableDefinitions >/dev/null 2>&1; \
	tail -n +5 dashboard/src/generated/client-model.ts > "$$tmp_dash_committed"; \
	diff -q "$$tmp_dash" "$$tmp_dash_committed" >/dev/null 2>&1 || { echo "ERROR: dashboard types out of sync. Run 'make generate-types'"; rm -f "$$tmp_dash" "$$tmp_dash_committed"; exit 1; }; \
	diff -q dashboard/src/generated/client-model.ts vscode-extension/src/generated/client-model.ts >/dev/null 2>&1 || { echo "ERROR: dashboard and vscode-extension types differ. Run 'make generate-types'"; rm -f "$$tmp_dash" "$$tmp_dash_committed"; exit 1; }; \
	rm -f "$$tmp_dash" "$$tmp_dash_committed"
	@tmp_rust=$$(mktemp); tmp_rust_committed=$$(mktemp); \
	python3 scripts/generate-rust-types.py $(SCHEMA) "$$tmp_rust" $(GEN_TIMESTAMP) >/dev/null 2>&1; \
	tail -n +5 "$$tmp_rust" > "$$tmp_rust.strip"; \
	tail -n +5 crates/skillpack-adapters/src/generated/client_model.rs > "$$tmp_rust_committed"; \
	diff -q "$$tmp_rust.strip" "$$tmp_rust_committed" >/dev/null 2>&1 || { echo "ERROR: Rust types out of sync. Run 'make generate-types'"; rm -f "$$tmp_rust" "$$tmp_rust_committed" "$$tmp_rust.strip"; exit 1; }; \
	rm -f "$$tmp_rust" "$$tmp_rust_committed" "$$tmp_rust.strip"
	@echo "All generated types are in sync with schema."

.PHONY: check-schema-drift check-proto-sync

check-schema-drift:
	@echo "Checking for local type definitions shadowing canonical types..."
	@python3 scripts/check-drift.py

check-proto-sync:
	@echo "Checking proto consolidation..."
	@if [ -f "macos/skills-ecosystem/SkillsDaemon/Sources/SkillsDaemon/canonical.proto" ]; then echo "ERROR: Stale canonical.proto found in SkillsDaemon"; exit 1; fi
	@if [ -f "macos/skills-ecosystem/SkillsCLI/Sources/SkillsCLI/canonical.proto" ]; then echo "ERROR: Stale canonical.proto found in SkillsCLI"; exit 1; fi
	@if [ -f "macos/skills-ecosystem/SkillsDaemon/Sources/SkillsDaemon/Skills.proto" ]; then echo "ERROR: Stale Skills.proto found in SkillsDaemon"; exit 1; fi
	@if [ -f "macos/skills-ecosystem/SkillsCLI/Sources/SkillsCLI/Skills.proto" ]; then echo "ERROR: Stale Skills.proto found in SkillsCLI"; exit 1; fi
	@echo "  No stale proto copies in macOS packages."
	@if find . -name "index.proto" -not -path "*/proto/*" -not -path "*/.build/*" -not -path "*/target/*" | grep -q .; then echo "ERROR: Stale index.proto found outside proto/"; exit 1; fi
	@if find . -name "query.proto" -not -path "*/proto/*" -not -path "*/.build/*" -not -path "*/target/*" | grep -q .; then echo "ERROR: Stale query.proto found outside proto/"; exit 1; fi
	@if find . -name "ratings.proto" -not -path "*/proto/*" -not -path "*/.build/*" -not -path "*/target/*" | grep -q .; then echo "ERROR: Stale ratings.proto found outside proto/"; exit 1; fi
	@echo "  No stale split proto copies."
	@cd macos/skills-ecosystem && $(MAKE) check-proto-sync
	@echo "Proto consolidation verified."
