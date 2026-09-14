SHELL = /usr/bin/env bash -o pipefail
.SHELLFLAGS = -ec

.PHONY: all
all: build

##@ General

# The help target prints out all targets with their descriptions organized
# beneath their categories. The categories are represented by '##@' and the
# target descriptions by '##'. The awk command is responsible for reading the
# entire set of makefiles included in this invocation, looking for lines of the
# file as xyz: ## something, and then pretty-format the target and help. Then,
# if there's a line with ##@ something, that gets pretty-printed as a category.
# More info on the usage of ANSI control characters for terminal formatting:
# https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters
# More info on the awk command:
# http://linuxcommand.org/lc3_adv_awk.php

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

.PHONY: build
build: ## Build the occonfig binary (release).
	# Build the release binary the same way a user would: `cargo build --release`.
	cargo build --release

.PHONY: check
check: ## Compile-check the workspace without producing a binary.
	cargo check --all-targets

.PHONY: fmt
fmt: ## Format the code with rustfmt.
	cargo fmt

.PHONY: fmt-check
fmt-check: ## Check formatting without modifying files (fails on drift).
	cargo fmt --check

.PHONY: lint
lint: ## Lint with clippy, warnings as errors.
	cargo clippy --all-targets -- -D warnings

.PHONY: install
install: build ## Install occonfig to /usr/local/bin (requires sudo).
	sudo install -m 0755 target/release/occonfig /usr/local/bin/occonfig

.PHONY: clean
clean: ## Remove build artifacts.
	cargo clean

##@ Testing

.PHONY: test
test: ## Run the full test suite (unit + integration).
	cargo test

.PHONY: test-unit
test-unit: ## Run unit tests only.
	cargo test --lib

.PHONY: test-cli
test-cli: ## Run integration tests only (tests/cli.rs).
	cargo test --test cli

.PHONY: test-cover
test-cover: ## Run tests with coverage instrumentation, writing lcov.info.
	# Requires: cargo install cargo-llvm-cov && rustup component add llvm-tools-preview
	cargo llvm-cov --workspace --lcov --output-path lcov.info

##@ Release

.PHONY: release-check
release-check: ## Validate the GoReleaser config against its schema.
	goreleaser check

.PHONY: release-snapshot
release-snapshot: ## Build a local snapshot release (writes dist/checksums.txt).
	goreleaser release --snapshot --clean

.PHONY: homebrew-dry-run
homebrew-dry-run: ## Render the Homebrew formula from dist/checksums.txt without publishing.
	# Run `make release-snapshot` first to produce dist/checksums.txt.
	DRY_RUN=1 ./hack/publish-homebrew-formula.sh 0.0.0-snapshot dist/checksums.txt

##@ CI Parity

.PHONY: ci
ci: fmt-check lint test ## Run everything CI runs, in order.
