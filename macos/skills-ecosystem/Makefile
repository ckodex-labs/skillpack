# Skills Ecosystem Makefile
# Automates compiling, generating Xcode targets, and registering macOS system services.
#
# NOTE: This project has been migrated to delegate canonical store logic to the
# Rust skillpack-server. The macOS layer is now a thin client.

PREFIX ?= /usr/local/bin
DAEMON_PLIST = com.ckodex.skillsdaemon.plist
LAUNCHD_DIR = $(HOME)/Library/LaunchAgents
SKILLPACK_ROOT ?= ../..

# Protoc plugin binaries (built from SPM checkouts)
PROTOC_GEN_SWIFT ?= $(SKILLPACK_ROOT)/macos/skills-ecosystem/SkillsDaemon/.build/checkouts/swift-protobuf/.build/arm64-apple-macosx/release/protoc-gen-swift
PROTOC_GEN_GRPC_SWIFT ?= $(SKILLPACK_ROOT)/macos/skills-ecosystem/SkillsDaemon/.build/checkouts/grpc-swift/.build/arm64-apple-macosx/release/protoc-gen-grpc-swift

.PHONY: all build build-rust generate generate-protos clean install-cli install-daemon install-extension uninstall

all: build-rust build generate

build-rust:
	@echo "Building Rust skillpack-server..."
	cd $(SKILLPACK_ROOT) && cargo build --release -p skillpack-api

build:
	@echo "Building Swift Packages..."
	cd SkillsCore && swift build -c release
	cd SkillsCLI && swift build -c release
	cd SkillsDaemon && swift build -c release

generate:
	@echo "Generating Xcode targets via XcodeGen..."
	xcodegen generate

generate-protos:
	@echo "Generating Swift gRPC stubs from canonical.proto..."
	protoc \
		--plugin=protoc-gen-swift=$(PROTOC_GEN_SWIFT) \
		--plugin=protoc-gen-grpc-swift=$(PROTOC_GEN_GRPC_SWIFT) \
		--swift_out=SkillsCLI/Sources/SkillsCLI \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsCLI/Sources/SkillsCLI \
		--grpc-swift_opt=Visibility=Public \
		--swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--grpc-swift_opt=Visibility=Public \
		-I $(SKILLPACK_ROOT)/proto/skillpack/v1 \
		$(SKILLPACK_ROOT)/proto/skillpack/v1/canonical.proto
	@echo "Generating Swift protobuf stubs from skills.proto..."
	protoc \
		--plugin=protoc-gen-swift=$(PROTOC_GEN_SWIFT) \
		--plugin=protoc-gen-grpc-swift=$(PROTOC_GEN_GRPC_SWIFT) \
		--swift_out=SkillsCLI/Sources/SkillsCLI \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsCLI/Sources/SkillsCLI \
		--grpc-swift_opt=Visibility=Public \
		--swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--grpc-swift_opt=Visibility=Public \
		-I $(SKILLPACK_ROOT)/proto/macos/v1 \
		$(SKILLPACK_ROOT)/proto/macos/v1/skills.proto
	@echo "Generating Swift stubs from skillpack.proto..."
	protoc \
		--plugin=protoc-gen-swift=$(PROTOC_GEN_SWIFT) \
		--plugin=protoc-gen-grpc-swift=$(PROTOC_GEN_GRPC_SWIFT) \
		--swift_out=SkillsCLI/Sources/SkillsCLI \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsCLI/Sources/SkillsCLI \
		--grpc-swift_opt=Visibility=Public \
		--swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--swift_opt=Visibility=Public \
		--grpc-swift_out=SkillsDaemon/Sources/SkillsDaemon \
		--grpc-swift_opt=Visibility=Public \
		-I $(SKILLPACK_ROOT)/proto/skillpack/v1 \
		$(SKILLPACK_ROOT)/proto/skillpack/v1/skillpack.proto
	@echo "Proto generation complete."

check-proto-sync:
	@echo "Checking proto consolidation..."
	@if [ -f "SkillsCLI/Sources/SkillsCLI/Skills.proto" ]; then echo "ERROR: Local Skills.proto found in SkillsCLI"; exit 1; fi
	@if [ -f "SkillsDaemon/Sources/SkillsDaemon/Skills.proto" ]; then echo "ERROR: Local Skills.proto found in SkillsDaemon"; exit 1; fi
	@echo "  No local proto copies."
	@if ! grep -q "../../../../../proto/macos/v1" SkillsCLI/Sources/SkillsCLI/swift-protobuf-config.json; then echo "ERROR: SkillsCLI swift-protobuf-config.json does not reference central proto"; exit 1; fi
	@if ! grep -q "../../../../../proto/macos/v1" SkillsDaemon/Sources/SkillsDaemon/swift-protobuf-config.json; then echo "ERROR: SkillsDaemon swift-protobuf-config.json does not reference central proto"; exit 1; fi
	@echo "  SPM plugin configs reference central proto directory."
	@echo "Proto consolidation verified."

generate-types:
	@echo "Swift types are maintained manually in SkillsCore/Sources/SkillsCore/Generated/ClientModel.swift"
	@echo "Regenerate by updating the file to match schemas/skills-specs-next/client-model.schema.json"
	@echo "Quicktype JSON Schema support is limited; manual maintenance is preferred for correctness."
	@echo "Remember to update the header timestamp to: $(GEN_TIMESTAMP)"

clean:
	@echo "Cleaning build artifacts..."
	rm -rf SkillsCore/.build SkillsCLI/.build SkillsDaemon/.build
	rm -rf SkillsEcosystem.xcodeproj
	rm -rf run-demo.sh.log demo-sandbox

install-cli:
	@echo "Building skills-cli (full ASM-parity CLI)..."
	mkdir -p $(PREFIX)
	cd SkillsCLI && swift build -c release
	cp SkillsCLI/.build/release/SkillsCLI $(PREFIX)/skills-cli
	@echo "skills-cli installed to $(PREFIX)/skills-cli"
	@echo "Run: skills-cli --help"

test-core:
	@echo "Running SkillsCore unit tests..."
	cd SkillsCore && swift test

install-daemon: build
	@echo "Installing daemon binary to $(PREFIX)..."
	mkdir -p $(PREFIX)
	cp SkillsDaemon/.build/release/SkillsDaemon $(PREFIX)/skills-daemon
	@echo "Registering background daemon service via launchd..."
	mkdir -p $(LAUNCHD_DIR)
	cp $(DAEMON_PLIST) $(LAUNCHD_DIR)/$(DAEMON_PLIST)
	# Bootstrap the agent
	launchctl unload $(LAUNCHD_DIR)/$(DAEMON_PLIST) 2>/dev/null || true
	launchctl load $(LAUNCHD_DIR)/$(DAEMON_PLIST)
	@echo "Daemon loaded! Monitoring background events."

install-extension: generate
	@echo "Building and registering Finder extension..."
	xcodebuild -project SkillsEcosystem.xcodeproj -scheme SkillsFinderSync -configuration Release
	@echo ""
	@echo "=== Extension Registration Completed ==="
	@echo "Please enable the extension manually in System Settings:"
	@echo "System Settings -> Privacy & Security -> Extensions -> Finder -> Enable 'SkillsFinderSync'"
	@echo "========================================"

uninstall:
	@echo "Uninstalling services..."
	launchctl unload $(LAUNCHD_DIR)/$(DAEMON_PLIST) 2>/dev/null || true
	rm -f $(LAUNCHD_DIR)/$(DAEMON_PLIST)
	rm -f $(PREFIX)/skills-cli
	rm -f $(PREFIX)/skills-daemon
	@echo "Uninstall complete."
