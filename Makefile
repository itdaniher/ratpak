CC=~/projects/rust/x86_64-unknown-linux-gnu/stage2/bin/rustc

all: stage3.rs

libabstrast*.so:
	$(CC) --crate-type=dylib abstrast.rs

stage2: libabstrast*.so
	$(CC) -L./ stage2.rs

stage3.rs: stage2
	rm -f stage2.json
	coffee stage1.coffee 0 > stage2.json
	./stage2 > stage3.rs

stage3: stage3.rs
	$(CC) -O -L../lib/ stage3.rs

clean:
	rm -f stage2 stage3 stage2.json stage3.rs
