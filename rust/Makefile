STEPS = step0_repl step1_read_print

.PHONY: all
all: build

.PHONY: $(STEPS)
$(STEPS): build

.PHONY: build
build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean
