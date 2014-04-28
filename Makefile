all: clean stage3

stage2.json:
	coffee stage1.coffee 0 > stage2.json

libabstrast.so:
	rustc -O --crate-type=dylib abstrast.rs

stage2: libabstrast.so
	rustc -O -L./ stage2.rs

stage3.rs: stage2 stage2.json
	./stage2 > stage3.rs

stage3: stage3.rs
	rustc -O -L../lib/ stage3.rs

clean:
	rm -f libabstrast*.so
	rm -f stage2 stage3
	rm -f stage2.json stage3.rs
