PREFIX ?= $(HOME)/.local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share/npushell

.PHONY: build install uninstall clean

build:
	cargo build --release

install: build
	@mkdir -p $(BINDIR) $(DATADIR)
	cp target/release/npushell $(BINDIR)/npushell
	cp shell/hooks.bash $(DATADIR)/hooks.bash
	cp shell/hooks.zsh $(DATADIR)/hooks.zsh
	cp shell/bash-preexec.sh $(DATADIR)/bash-preexec.sh
	@echo ""
	@echo "npushell installed successfully!"
	@echo ""
	@echo "Add one of the following to your shell config:"
	@echo ""
	@echo "  For bash (~/.bashrc):"
	@echo "    source $(DATADIR)/hooks.bash"
	@echo ""
	@echo "  For zsh (~/.zshrc):"
	@echo "    source $(DATADIR)/hooks.zsh"
	@echo ""
	@echo "Then restart your shell or run: source ~/.bashrc (or ~/.zshrc)"
	@echo "Run 'npushell doctor' to verify your setup."

uninstall:
	rm -f $(BINDIR)/npushell
	rm -rf $(DATADIR)
	@echo "npushell uninstalled. Remember to remove the source line from your shell config."

clean:
	cargo clean
