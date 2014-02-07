RUSTFLAGS ?=
OUTDIR ?= build

BINDIR := $(OUTDIR)/bin
DOCDIR := $(OUTDIR)/doc
LIBDIR := $(OUTDIR)/lib
TMPDIR := $(OUTDIR)/tmp

RUST_SRC := $(shell find src -type f -name '*.rs')

.PHONY: all
all: $(TMPDIR)/librose.dummy doc

$(BINDIR) $(DOCDIR) $(LIBDIR) $(TMPDIR):
	mkdir -p '$@'

$(TMPDIR)/librose.dummy: src/lib.rs $(RUST_SRC) $(LIBDIR) $(TMPDIR)
	rustc --out-dir '$(LIBDIR)' --crate-type=lib src/lib.rs $(RUSTFLAGS)
	touch $@

.PHONY: doc
doc:
	rustdoc src/lib.rs -o '$(DOCDIR)'

.PHONY: clean
clean:
	rm -fr '$(OUTDIR)'
