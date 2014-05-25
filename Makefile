all: stage3.rs

CC = rustc

libabstrast*.so:
	$(CC) --crate-type=dylib abstrast.rs

stage2: libabstrast*.so
	$(CC) -L./ stage2.rs

stage3.rs: stage2
	rm -f stage2.json
	coffee stage1.coffee 0 > stage2.json
	./stage2
	mkdir dots
	mv *.dot dots

stage3: stage3.rs
	$(CC) -O -L../lib/ stage3.rs

clean:
	rm -f stage2 stage3 stage2.json stage3.rs
	rm -rf dots
