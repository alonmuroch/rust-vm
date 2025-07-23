# Makefile to build riscv-tests for RISC-V VM compliance

# riscv-tests is a submodule of this repository; no need to clone.
RISCV_TESTS_DIR := riscv-tests
INSTALL_PREFIX ?= $(CURDIR)/riscv-tests-install

.PHONY: all full update-submodules autogen configure build install clean

# Update submodules (if already cloned)
update-submodules:
	cd $(RISCV_TESTS_DIR) && git submodule update --init --recursive

# Generate configure script
autogen:
	cd $(RISCV_TESTS_DIR) && autoconf

# Configure the build
configure:
	cd $(RISCV_TESTS_DIR) && ./configure --prefix=$(INSTALL_PREFIX)

# Build the tests
build:
	cd $(RISCV_TESTS_DIR) && make

# Install the tests
install:
	cd $(RISCV_TESTS_DIR) && make install

# Full flow: update submodules, autogen, configure, build, install
all: update-submodules autogen configure build install

# 'full' target: run everything in one go (recommended for first-time setup)
full: update-submodules autogen configure build install

clean:
	cd $(RISCV_TESTS_DIR) && make clean || true
	 rm -rf $(INSTALL_PREFIX) 