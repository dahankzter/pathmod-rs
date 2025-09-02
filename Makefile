# Coverage helpers for the Rust workspace
# Requires: cargo (stable), llvm-tools component. We auto-install cargo-llvm-cov if missing.

# You can override CARGO and RUSTUP if needed
CARGO ?= cargo
RUSTUP ?= rustup

# Directory where cargo-llvm-cov puts reports by default
COV_DIR := target/llvm-cov
HTML_INDEX := $(COV_DIR)/html/index.html
LCOV_FILE := $(COV_DIR)/lcov.info

.PHONY: coverage coverage-html coverage-lcov coverage-summary coverage-clean coverage-open ensure-llvm-tools ensure-llvm-cov

# Main coverage target: text summary and HTML report
coverage: ensure-llvm-tools ensure-llvm-cov
	$(CARGO) llvm-cov clean --workspace
	$(CARGO) llvm-cov --workspace --all-features --html
	@echo "HTML coverage report: $(HTML_INDEX)"

# Print a concise terminal summary without generating HTML
coverage-summary: ensure-llvm-tools ensure-llvm-cov
	$(CARGO) llvm-cov clean --workspace
	$(CARGO) llvm-cov --workspace --all-features

# Explicit HTML target alias
coverage-html: coverage

# Generate lcov.info for CI services that ingest LCOV
coverage-lcov: ensure-llvm-tools ensure-llvm-cov
	$(CARGO) llvm-cov clean --workspace
	$(CARGO) llvm-cov --workspace --all-features --lcov --output-path $(LCOV_FILE)
	@echo "LCOV report: $(LCOV_FILE)"

# Remove coverage artifacts
coverage-clean:
	@rm -rf $(COV_DIR)

# Try to open the HTML report in a browser (macOS or Linux)
coverage-open:
	@if [ -f "$(HTML_INDEX)" ]; then \
	  if command -v open >/dev/null 2>&1; then open "$(HTML_INDEX)"; \
	  elif command -v xdg-open >/dev/null 2>&1; then xdg-open "$(HTML_INDEX)"; \
	  else echo "Open $(HTML_INDEX) in your browser"; fi; \
	else \
	  echo "No HTML report found. Run 'make coverage' first."; \
	fi

# Ensure rustup component and tool are present
ensure-llvm-tools:
	@$(RUSTUP) component add llvm-tools-preview >/dev/null 2>&1 || true

ensure-llvm-cov:
	@$(CARGO) llvm-cov --version >/dev/null 2>&1 || $(CARGO) install cargo-llvm-cov >/dev/null 2>&1 || (echo "Failed to install cargo-llvm-cov. Install manually: 'cargo install cargo-llvm-cov'" && exit 1)
