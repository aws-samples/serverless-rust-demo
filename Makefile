STACK_NAME ?= rust-products
FUNCTIONS := get-products get-product put-product delete-product

export CC_aarch64_unknown_linux_gnu = aarch64-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = aarch64-linux-gnu-gcc
STRIP := aarch64-linux-gnu-strip

.PHONY: build deploy tests

all: build deploy tests

build:
	cargo build --release --target aarch64-unknown-linux-gnu
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${FUNCTIONS}, build-${function})

build-%:
	mkdir -p ./build/$*
	$(STRIP) ./target/aarch64-unknown-linux-gnu/release/$*
	cp -v ./target/aarch64-unknown-linux-gnu/release/$* ./build/$*/bootstrap
	cp -v ./otel-config.yaml ./build/$*/collector.yaml

deploy:
	if [ -f samconfig.toml ]; \
		then sam deploy --stack-name $(STACK_NAME); \
		else sam deploy -g --stack-name $(STACK_NAME); \
	fi

tests:
	RUST_BACKTRACE=1 REST_API=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
		--output text) cargo test