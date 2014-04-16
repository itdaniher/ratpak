all: clean stage3

stage2.json:
	coffee stage1.coffee 2 > stage2.json

libast.so:
	rustc -O --crate-type=dylib ast.rs

stage2: libast.so
	rustc -O -L./ stage2.rs

stage3.rs: stage2 stage2.json
	./stage2 > stage3.rs

stage3: stage3.rs
	rustc -O -L../lib/ stage3.rs

clean:
	rm -f libast*.so
	rm -f stage2
	rm -f stage2.json stage3.rs
