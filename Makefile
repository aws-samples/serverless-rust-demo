STACK_NAME ?= rust-products
FUNCTIONS := get-products get-product put-product delete-product dynamodb-streams

ARCH := aarch64-unknown-linux-gnu
ARCH_SPLIT = $(subst -, ,$(ARCH))

.PHONY: build deploy tests

all: build tests-unit deploy tests-integ
ci: build tests-unit

setup:
ifeq (,$(shell which rustc))
	$(error "Could not found Rust compiler, please install it")
endif
ifeq (,$(shell which cargo))
	$(error "Could not found Cargo, please install it")
endif
ifeq (,$(shell which zig))
	$(error "Could not found Zig compiler, please install it")
endif
	cargo install cargo-zigbuild
ifeq (,$(shell which sam))
	$(error "Could not found SAM CLI, please install it")
endif
ifeq (,$(shell which artillery))
	$(error "Could not found Artillery, it's required for load testing")
endif

build:
ifeq ("$(shell zig targets | jq -r .native.cpu.arch)-$(shell zig targets | jq -r .native.os)-$(shell zig targets | jq -r .native.abi)", "$(word 1,$(ARCH_SPLIT))-$(word 3,$(ARCH_SPLIT))-$(word 4,$(ARCH_SPLIT))")
	@echo "Same host and target. Using native build"
	cargo build --release --target $(ARCH)
else
	@echo "Different host and target. Using zigbuild"
	cargo zigbuild --release --target $(ARCH)
endif
	
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${FUNCTIONS}, build-${function})

build-%:
	mkdir -p ./build/$*
	cp -v ./target/$(ARCH)/release/$* ./build/$*/bootstrap

deploy:
	if [ -f samconfig.toml ]; \
		then sam deploy --stack-name $(STACK_NAME); \
		else sam deploy -g --stack-name $(STACK_NAME); \
	fi

tests-unit:
	cargo test --lib --bins

tests-integ:
	RUST_BACKTRACE=1 API_URL=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
		--output text) cargo test

tests-load:
	API_URL=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`ApiUrl`].OutputValue' \
		--output text) artillery run tests/load-test.yml
