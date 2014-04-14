all: temps

temps.json:
	coffee test.coffee

libast.so:
	rustc -O --crate-type=dylib ast.rs

instantGen: libast.so
	rustc -O -L./ instantGen.rs

temps.rs: temps.json instantGen
	./instantGen > temps.rs

temps: temps.rs
	rustc -O -L../lib/ temps.rs

clean:
	rm libast*.so
	rm instantGen
	rm temps.json temps
