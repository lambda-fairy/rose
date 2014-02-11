RUSTFLAGS ?=
OUTDIR ?= build

BINDIR := $(OUTDIR)/bin
DOCDIR := $(OUTDIR)/doc
LIBDIR := $(OUTDIR)/lib
TMPDIR := $(OUTDIR)/tmp

RUST_SRC := $(shell find src -type f -name '*.rs')

.PHONY: all
all: lib doc

$(BINDIR) $(DOCDIR) $(LIBDIR) $(TMPDIR):
	mkdir -p '$@'

.PHONY: doc
doc: $(DOCDIR)
	rustdoc src/lib.rs -o '$(DOCDIR)'

.PHONY: lib
lib: $(TMPDIR)/librose.dummy

$(TMPDIR)/librose.dummy: src/lib.rs $(RUST_SRC) $(LIBDIR) $(TMPDIR)
	rustc --out-dir '$(LIBDIR)' --crate-type=lib src/lib.rs $(RUSTFLAGS)
	touch $@

.PHONY: test
test: src/lib.rs $(RUST_SRC) $(TMPDIR)
	rustc --test -o '$(TMPDIR)/rose-test' src/lib.rs $(RUSTFLAGS)
	'$(TMPDIR)/rose-test'

.PHONY: clean
clean:
	rm -fr '$(OUTDIR)'
