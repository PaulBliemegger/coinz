.PHONY: setup dev backend frontend quickcheck check format hooks browser-install

setup:
	pnpm --dir frontend install --frozen-lockfile

dev:
	./scripts/dev.sh

backend:
	COINZ_ORIGIN=http://localhost:5173 cargo run -- serve

frontend:
	pnpm --dir frontend dev

quickcheck:
	cargo fmt --all --check
	pnpm --dir frontend format:check
	pnpm --dir frontend lint

check: quickcheck
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo test --workspace --locked
	pnpm --dir frontend check
	pnpm --dir frontend test
	pnpm --dir frontend build
	cargo build --release --locked
	pnpm --dir frontend test:e2e

format:
	cargo fmt --all
	pnpm --dir frontend format

hooks:
	git rev-parse --git-dir >/dev/null
	git config core.hooksPath .githooks
	@echo 'Installed coinz pre-commit checks.'

browser-install:
	pnpm --dir frontend exec playwright install chromium
