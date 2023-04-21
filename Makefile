test:
	cargo test
.PHONY: test

test-all-examples:
	set -eux -o pipefail; \
	for dir in ./example_crates/*/; do \
		(cd $$dir && cargo test); \
	done

.PHONY: test-all-examples
