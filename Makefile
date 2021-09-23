export STACK_NAME=rust-products
export FUNCTIONS=get-products get-product put-product delete-product

.PHONY: build deploy tests

all: build deploy tests

build:
	CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc cargo build --release --target x86_64-unknown-linux-musl
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${FUNCTIONS}, build-${function})

build-mac:
	CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${FUNCTIONS}, build-${function})


build-%:
	mkdir -p ./build/$*
	strip ./target/x86_64-unknown-linux-musl/release/$*
	cp -v ./target/x86_64-unknown-linux-musl/release/$* ./build/$*/bootstrap
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