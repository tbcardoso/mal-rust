STEPS = step0_repl step1_read_print step2_eval step3_env step4_if_fn_do step5_tco

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
