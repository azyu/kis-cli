RUST_MANIFEST := rust/Cargo.toml

.PHONY: fmt fmt-check lint test hooks-install

fmt:
	cargo fmt --manifest-path $(RUST_MANIFEST) --all

fmt-check:
	cargo fmt --manifest-path $(RUST_MANIFEST) --all --check

lint:
	cargo clippy --manifest-path $(RUST_MANIFEST) --workspace --all-targets -- -D warnings

test:
	cargo test --manifest-path $(RUST_MANIFEST)

hooks-install:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit .githooks/pre-push
