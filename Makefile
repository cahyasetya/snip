.PHONY: build run test clean install release-patch release-minor release-major

build:
	cargo build

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

install:
	cargo install --path .

release-patch:
	gh workflow run cargo-release.yml -f level=patch

release-minor:
	gh workflow run cargo-release.yml -f level=minor

release-major:
	gh workflow run cargo-release.yml -f level=major
