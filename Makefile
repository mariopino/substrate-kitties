.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run
run:
	cargo run --release -- --dev --tmp

.PHONY: run-debug
run-debug:
	cargo run -- --dev --tmp -lruntime=debug -lpallet_kitties=debug --unsafe-ws-external

.PHONY: build
build:
	cargo build --release
