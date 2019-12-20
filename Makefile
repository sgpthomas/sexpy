.PHONY: test expand

expand:
	cargo expand --lib --tests

test:
	cargo test -- --nocapture
