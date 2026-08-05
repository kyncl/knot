BINARY_NAME=knot
PREFIX=$(HOME)/.local/bin

.PHONY: all build install clean uninstall help

all: build

help:
	@echo "Available targets: build, install, uninstall, clean"

build:
	cargo build --release

install: build
	@test -f target/release/$(BINARY_NAME) || (echo "Error: Binary not found"; exit 1)
	@echo "Installing $(BINARY_NAME) to $(PREFIX)..."
	@install -Dm755 target/release/$(BINARY_NAME) $(PREFIX)/$(BINARY_NAME)
	@echo "Successfully installed $(BINARY_NAME)!"
	@echo "If you want shell complete for $(BINARY_NAME), use $(BINARY_NAME) complete --help"

uninstall:
	@echo "Removing $(BINARY_NAME) from $(PREFIX)..."
	@rm -f $(PREFIX)/$(BINARY_NAME)
	@echo "Successfully uninstalled $(BINARY_NAME)!"

clean:
	cargo clean
