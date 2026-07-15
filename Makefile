.PHONY: setup format format-check lint test build check desktop-dev service-dev

setup:
	cd desktop && npm ci

format:
	cargo fmt --all
	cd desktop && npm run format

format-check:
	cargo fmt --all -- --check
	cd desktop && npm run format:check

lint:
	cargo lint
	cd desktop && npm run lint

test:
	cargo test-all

build:
	cargo build --workspace
	cd desktop && npm run build

check: format-check lint test
	cd desktop && npm run typecheck && npm run build

desktop-dev:
	cd desktop && npm run tauri dev

service-dev:
	cd desktop && npm run dev:service

.PHONY: test-layout
test-layout:
	python3 scripts/check-rust-test-layout.py
