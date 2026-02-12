PYTHON ?= python3
PIP ?= $(PYTHON) -m pip
RUFF ?= ruff
PYTEST ?= pytest
MATURIN ?= maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY ?= 1

.PHONY: lint lint-check format format-rust test install build deps-build
.PHONY: benchmark

lint: deps-build
	$(RUFF) format debot/
	cargo fmt --manifest-path rust/Cargo.toml

deps-build:
	$(PIP) install ruff
	$(PIP) install maturin

build: deps-build
	PYO3_USE_ABI3_FORWARD_COMPATIBILITY=$(PYO3_USE_ABI3_FORWARD_COMPATIBILITY) \
		$(MATURIN) build --release -m rust/Cargo.toml

install:
	PYO3_USE_ABI3_FORWARD_COMPATIBILITY=$(PYO3_USE_ABI3_FORWARD_COMPATIBILITY) \
		$(PIP) install .

test: build
	@WHEEL=$$(ls -1t rust/target/wheels/*.whl | head -n 1); \
	PYO3_USE_ABI3_FORWARD_COMPATIBILITY=$(PYO3_USE_ABI3_FORWARD_COMPATIBILITY) \
		$(PIP) install $$WHEEL
	$(PIP) install ".[dev]"
	$(PYTEST) tests/ -v --tb=short

benchmark:
	$(PIP) install -r benchmarks/requirements.txt
	$(PYTHON) benchmarks/run_all.py --max-samples 5

benchmark-router: build
	@WHEEL=$$(ls -1t rust/target/wheels/*.whl | head -n 1); \
	$(PIP) install $$WHEEL
	$(PIP) install -r benchmarks/requirements.txt
	$(PYTHON) benchmarks/router_savings.py --max-samples 50
