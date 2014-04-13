extern crate serialize;
use std::io::File;
use std::path::Path;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable,Encodable)]
enum Args {
	Float(f64),
	String(~str),
	Vec(~[Args])
}

#[deriving(Decodable,Encodable, Clone)]
struct Node {
	pname: ~str,
	oct: uint,
	ict: uint,
	args: Option<~str>
}


#[deriving(Decodable,Encodable)]
struct Graph {
	edges: ~[~[~str]],
	nodes: ~[(~str, Node)]
}

fn main () {
	let json_str_to_decode = File::open(&Path::new("./temps.json")).read_to_str().unwrap();
	let json_object = json::from_str(json_str_to_decode.to_owned());
	let mut decoder = json::Decoder::new(json_object.unwrap());
	let y: Graph = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
		Err(e) => fail!("Decoding error: {}", e)
	};
	
	if ( y.nodes.len() & y.edges.len() ) > 0 {
		for (uid, node) in y.nodes.clone().move_iter() {
			let mut rxers: ~[~str] = ~[];
			let mut txers: ~[~str] = ~[];
			for edge in y.edges.iter() {
				if uid == edge[0] {
					let mut e = ~"tx";
					e.push_str(edge[0].slice_from(0));
					e.push_str(edge[1].slice_from(0));
					txers.push(e);
				}
				else if uid == edge[1] {
					let mut e = ~"rx";
					e.push_str(edge[0]);
					e.push_str(edge[1]);
					rxers.push(e);
				}
				else {
				}
			}
	println!("{:?}", (txers,rxers));
		}
	}
}
