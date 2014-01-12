all: rose doc

clean:
	rm -fr build/
	rm -fr doc/

doc:
	cd src/rose && rustdoc lib.rs -o ../../doc

# rustpkg tracks dependencies automatically, so we don't need to
# list sources here
rose:
	rustpkg build rose

.PHONY: all clean doc rose
