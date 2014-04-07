extern crate msgpack;
use msgpack::{String, Array, Map, Unsigned, Double, Value, Nil};
use std::str::eq;
use std::vec;

fn main () {
	let x = include_bin!("temps.msgpack");
	let y: msgpack::Value = msgpack::from_msgpack(x.to_owned());
	let mut edges = ~[];
	let mut nodes = ~[];
	match y {
		Map(x) => for v in x.move_iter() {
			match v {
				(String(a),Array(b)) => match a.slice_from(0) {
					"edges" => {edges = unpackEdges(b);},
					"nodes" => {nodes = unpackNodes(b);},
					_ => (),
				},
				(a,b) => println!("{:?}", (a,b))
			}},
		_ => ()
	};
	if ( nodes.len() & edges.len() ) > 0 {
		for node in nodes.iter() {
			let mut rxers = ~[];
			let mut txers = ~[];
			for edge in edges.iter() {
				if node.uid == edge[0] {
					let mut e = ~"tx";
					e.push_str(edge[0].slice_from(0));
					e.push_str(edge[1].slice_from(0));
					txers.push(e);
				}
				if node.uid == edge[1] {
						let mut e = ~"rx";
						e.push_str(edge[0]);
						e.push_str(edge[1]);
						rxers.push(e);
				}
			}
			println!("{:?}", (txers,rxers));
		}
	}
}

fn unpackEdges(In: ~[Value]) -> ~[~[~str]] {
	In.move_iter().filter_map(|x| { match x { Array(a) => Some(a.move_iter().filter_map(|y| { match y { String(y) => Some(y), _ => None }}).collect()), _ => None }}).collect()
}

struct Node {
	uid: ~str,
	pname: ~str,
	label: ~str,
	oct: uint,
	ict: uint,
	args: ~[Value],
}

fn unpackNodes(In: ~[Value]) -> ~[Node] {
	let mut out = ~[];
	let mut In = In.move_iter();
	let mut uid = ~"";
	let mut pname = ~"";
	let mut label = ~"";
	let mut oct = 0u;
	let mut ict = 0u;
	'walk: loop {
		match In.next() {
			Some(Array(mut y)) => {
				let mut args = ~[];
				match (y.shift(), y.shift()) {
					(Some(String(a)), Some(Map(b))) => {
					uid = a;
					for c in b.move_iter() { match c {
						(String(d), String(e)) => match d.slice_from(0) {
							"proc" => {pname = e;}
							"label" => {label = e;}
							_ => ()
						},
						(String(d), Unsigned(e)) => match d.slice_from(0) {
							"ict" => {ict = e as uint;}
							"oct" => {oct = e as uint;}
							_ => ()
							},
						(String(d), Array(e)) => match d.slice_from(0) {
							"args" => {args=e;}
							_ => ()
						},
						(String(d), Nil) => match d.slice_from(0) {
							"args" => {args=~[];},
							_ => ()
						},
						(d, e) => println!("{:?}", (d,e)),
					}};
				out.push(Node{uid: uid.clone(), pname: pname.clone(), label: label.clone(), oct: oct.clone(), ict: ict.clone(), args: args});
				}
				x => println!("{:?}", x)
				}},
			None => break 'walk,
			x => println!("{:?}", x)
		}
	}
	return out
}
