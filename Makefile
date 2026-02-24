PREFIX ?= /usr/local

.PHONY: build release install uninstall clean test bench

build:
	cargo build

release:
	cargo build --release

install: release
	install -Dm755 target/release/txt-deep-search $(DESTDIR)$(PREFIX)/bin/txt-deep-search

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/txt-deep-search

clean:
	cargo clean

test:
	cargo test

bench: release
	@echo "=== Benchmark: txt-deep-search vs grep ==="
	@echo "Creating test data..."
	@mkdir -p /tmp/tds_bench
	@for i in $$(seq 1 2000); do \
		echo "This is line one of file $$i" > /tmp/tds_bench/file_$$i.txt; \
		echo "The quick brown fox jumps over the lazy dog" >> /tmp/tds_bench/file_$$i.txt; \
		echo "Line with target keyword BENCHMARK_NEEDLE here" >> /tmp/tds_bench/file_$$i.txt; \
		echo "Another line with some random data $$RANDOM" >> /tmp/tds_bench/file_$$i.txt; \
	done
	@echo ""
	@echo "--- grep -rn (2000 files) ---"
	@time grep -rn "BENCHMARK_NEEDLE" /tmp/tds_bench/ > /dev/null 2>&1
	@echo ""
	@echo "--- txt-deep-search (2000 files) ---"
	@time ./target/release/txt-deep-search /tmp/tds_bench --query "BENCHMARK_NEEDLE" > /dev/null 2>&1
	@echo ""
	@rm -rf /tmp/tds_bench
	@echo "Benchmark complete."
