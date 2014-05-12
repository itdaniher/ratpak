#![feature(globs)]

extern crate serialize;
extern crate syntax;
extern crate abstrast;

use std::io::File;
use std::path::Path;
use abstrast::*;
use syntax::ast;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable,Encodable,Clone)]
struct Node {
	pname: ~str
}

// naive graph format
#[deriving(Decodable,Encodable)]
struct Graph {
	edges: Vec<Vec<~str>>,
	nodes: Vec<(~str, Node)>,
	name: ~str,
	args: Option<Vec<~str>>,
	inrx: bool,
	outtx: bool,
}

fn expandPrim(nodepname: ~str) -> ~str {
	match nodepname.char_at(0) {
		'*' => ~"mulAcross",
		'+' => ~"sumAcross",
		'Z' => ~"delay",
		'%' => ~"grapes",
		'B' => ~"binconv",
		'V' => ~"vec",
		'$' => ~"shaper",
		'?' => ~"matcher",
		'&' => ~"crossApplicator",
		'!' => ~"applicator",
		',' => expandPrim(nodepname.slice_from(1).to_owned()).append("Vecs"),
		 _  => nodepname
	}
}

fn getDefaultArgs(nodepname: ~str) -> ~str {
	match nodepname.len() {
		1 => match nodepname.char_at(0) {
			'*' => ~"1.0f32",
			'+' | 'Z' => ~"0.0f32",
			'?' => ~"|a, b| {a.map(|x| {match x {Some(y) => b.send(y), None => ()}}).last();()}",
			_ => ~"",
		},
		2 => match (nodepname.char_at(0), nodepname.char_at(1)) {
			(',', x) => "range(0,512).map(|_| ".to_owned()
				.append(getDefaultArgs(std::str::from_char(x))).append(").collect()"),
			_ => ~""
		},
		_ => ~""
	}
}

fn getGraph() -> Vec<(Graph, Vec<json::Json>)> {
	// get json graph and node arguments from stage2.json
	let json_str_to_decode = File::open(&Path::new("./stage2.json")).read_to_str().unwrap();
	let json_object = json::from_str(json_str_to_decode.to_owned()).unwrap();
	let mut decoder = json::Decoder::new(json_object.clone());
	// extract enum-guarded arguments from json, do not deguard
	let args: Vec<Vec<json::Json>> = json_object.as_list().unwrap().iter().map(|x| x.search(&~"nodes").unwrap().as_list().unwrap().iter()
		.map(|x| {x.as_list().unwrap()[1].find(&~"args").unwrap().clone()}).collect()).collect();
	let y: Vec<Graph> = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
		Err(e) => fail!("Decoding error: {}", e)
	};
	y.move_iter().zip(args.move_iter()).collect()
}


fn genFunction(g: Graph, args: Vec<json::Json>) -> ast::Item {
	let mut channelStmts: Vec<ast::P<ast::Stmt>> = vec!();
	let mut spawnExprs: Vec<ast::P<ast::Expr>> = vec!();
	let mut fnargv: Vec<ast::Arg> = vec!();
	let mut io: Vec<~str> = vec!();
	// this does the work - iterate over nodes and arguments
	for (uid, node) in g.nodes.clone().move_iter() {
		let mut rxers: Vec<~str> = vec!();
		let mut txers: Vec<~str> = vec!();
		let n = expandPrim(node.pname.clone());
		for edge in g.edges.iter() {
			if &uid == edge.get(0) {
				txers.push("tx".to_owned().append(edge.get(0).slice_from(0)).append(edge.get(1).slice_from(0)));
			}
			else if &uid == edge.get(1) {
				rxers.push("rx".to_owned().append(edge.get(0).slice_from(0)).append(edge.get(1).slice_from(0)));
			}
		}
		match n.slice_from(0) {
			"in" => {
				let x = (~"r").append(txers.get(0).slice_from(1));
				io.push(x.clone());
				fnargv.push(abstrast::arg(x, "Receiver<f32>"))
				}
			"out" => {
				let x = (~"t").append(rxers.get(0).slice_from(1));
				io.push(x.clone());
				fnargv.push(abstrast::arg(x, "Sender<f32>"))
				}
			_ => {}
		}
	}
	for ((uid, node), arg) in g.nodes.clone().move_iter().zip(args.move_iter()) {
		let mut rxers: Vec<~str> = vec!();
		let mut txers: Vec<~str> = vec!();
		for edge in g.edges.iter() {
			if &uid == edge.get(0) {
				let ename = "tx".to_owned().append(edge.get(0).slice_from(0)).append(edge.get(1).slice_from(0));
				txers.push(ename);
			}
			else if &uid == edge.get(1) {
				rxers.push("rx".to_owned().append(edge.get(0).slice_from(0)).append(edge.get(1).slice_from(0)));
			}
		}
		let n = expandPrim(node.pname.clone());
		let mut argv = vec!();
		match rxers.len() {
			0 => (),
			1 => argv.push(expr_path(rxers.get(0).slice_from(0))),
			_ => argv.push(expr_owned_vec(rxers.iter().map(|x| expr_path(x.slice_from(0))).collect()))
		}
		match txers.len() {
			0 => (),
			1 => argv.push(expr_path(txers.get(0).slice_from(0))),
			_ => {
					let ftx = txers.get(0).slice_to(13).to_str().append("0");
					let frx = (~"r").append(ftx.slice_from(1));
					spawnExprs.push(spawn(~"fork",
						vec!(expr_path(frx), expr_owned_vec(txers.iter().map(
							|x| expr_path(x.slice_from(0))).collect()))));
					println!("{}", (&io, &ftx, &txers));
						channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(ftx.clone()),
							pat_name(frx.clone()))), expr_call(expr_path("channel"), vec!())));
					argv.push(expr_path(ftx))
				}
			};

		argv.push_all_move(
			match JSONtoAST(arg.clone()) {
				Some(lits) => {
					match lits {
						ast::ExprVec(v) => if v.len() > 0 {v} else {
						match getDefaultArgs(node.pname.clone()).slice_from(0) {
							"" => vec!(),
							x => vec!(parse_expr(x))
						}},
						ast::ExprPath(_) | ast::ExprVstore(_, _) => vec!(expr(lits)),
						_ => fail!("{:?}", lits)
					}
				}
				None => vec!()
			});
		match n.slice_from(0) {
			"in" => {},
			"out" => {},
			_ => {
					spawnExprs.push(spawn(n, argv));
					txers.iter().map(|txer| {
						let dstrm = (~"r").append((*txer).slice_from(1));
						if io.clone().iter().filter(|x| x.slice_from(0) == txer.slice_from(0)).len() < 1 {
							channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(*txer),
								pat_name(dstrm))), expr_call(expr_path("channel"), vec!())));
							}
						}).last();
			}
		}
	};
	channelStmts.push_all_move(spawnExprs.move_iter().map(|x| stmt_semi(x)).collect());
	match g.args {
		Some(ref aargs) => {aargs.iter().map(|x| fnargv.push(abstrast::arg(x.slice_from(0), "f32"))).last();},
		None => ()
	};
	fn_item(g.name, fnargv, ty_nil(), block(channelStmts, None))
}

fn main () {
	let forest: Vec<(Graph, Vec<json::Json>)> = getGraph();
	println!("{}", File::open(&Path::new("./boilerplate.rs")).read_to_str().unwrap());
	let o: Vec<~str> = forest.move_iter().map(|(x,y)| genFunction(x,y)).map(|z| syntax::print::pprust::item_to_str(&z)).collect();
	for f in o.iter() {
		println!("{}", f);
	}
}
