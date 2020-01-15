.PHONY: test expand docs

expand:
	cargo expand --lib --tests

test:
	cargo test -- --nocapture

docs:
	cargo doc --no-deps

clean:
	cargo clean
