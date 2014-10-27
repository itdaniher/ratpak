#![feature(globs)]

extern crate serialize;
extern crate syntax;
extern crate abstrast;
extern crate graphviz;

use graphviz::maybe_owned_vec::IntoMaybeOwnedVector;
use std::string::String;
use std::io::File;
use std::path::Path;
use abstrast::*;
use syntax::ast;
use syntax::ptr;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable,Encodable,Clone)]
struct Node {
	pname: String,
	uid: String,
	label: String
}

type Edge = (String, String);

// naive graph format
#[deriving(Decodable,Encodable,Clone)]
struct Graph {
	edges: Vec<Edge>,
	nodes: Vec<Node>,
	name: String,
	args: Option<Vec<String>>,
	inrx: bool,
	outtx: bool,
}

impl<'a> graphviz::Labeller<'a, Node, Edge> for Graph {
    fn graph_id(&'a self) -> graphviz::Id<'a> { graphviz::Id::new(self.name.as_slice()) }
    fn node_id(&'a self, n: &Node) -> graphviz::Id<'a> {
        graphviz::Id::new(n.uid.clone().into_maybe_owned())
    }
    fn node_label<'a>(&'a self, n: &Node) -> graphviz::LabelText<'a> {
        graphviz::LabelStr(n.label.clone().into_maybe_owned())
    }
    fn edge_label<'a>(&'a self, _: &Edge) -> graphviz::LabelText<'a> {
        graphviz::LabelStr("".into_maybe_owned())
    }
}

impl<'a> graphviz::GraphWalk<'a, Node, Edge> for Graph {
    fn nodes(&'a self) -> graphviz::Nodes<'a,Node> {
        self.nodes.clone().into_maybe_owned()
    }
    fn edges(&'a self) -> graphviz::Edges<'a,Edge> {
        self.edges.clone().into_maybe_owned()
    }
    fn source(&self, e: &Edge) -> Node { let &(ref s,_) = e; self.nodes.iter().filter_map(|n| {if &n.uid == s { Some(n.clone()) } else {None}}).next().unwrap() }
    fn target(&self, e: &Edge) -> Node { let &(_,ref t) = e; self.nodes.iter().filter_map(|n| {if &n.uid == t { Some(n.clone()) } else {None}}).next().unwrap() }
}

fn expand_prim(nodepname: String) -> String {
	let mut snp = nodepname.clone();
	let c0 = snp.remove(0).unwrap();
	let mut out: String= "".to_string();
	match c0 {
		'*' => "mul",
		'+' => "sum",
		'Z' => "delay",
		'%' => "grapes",
		'B' => "binconv",
		'V' => "vec",
		'$' => "shaper",
		'~' => "softSource",
		'{' => "looper",
		'&' => "crossApplicator",
		'!' => "applicator",
		'?' => {out = expand_prim(snp); out.push_str("Optional"); out.as_slice()},
		'/' => {out = expand_prim(snp); out.push_str("Across"); out.as_slice()},
		',' => {out = expand_prim(snp); out.push_str("Vecs"); out.as_slice()},
		 _  => nodepname.as_slice()
	}.to_string()
}

fn get_default_args(nodepname: String) -> String{
	let mut snp = nodepname.clone();
	let c0 = snp.remove(0).unwrap();
	let mut out: String= "".to_string();
	match nodepname.len() {
		1...3 => match c0 {
			'*' => "num::one()",
			'+' | 'Z' => "num::zero()",
			'/' => { out = get_default_args(snp); out.as_slice()}
			',' => { match get_default_args(snp).as_slice() {
					"" => "",
					x => {out = "range(0,512).map(|_| ".to_string(); out.push_str(x); out.push_str(").collect()"); out.as_slice()}
				}
			}
			_ => "",
		},
		_ => ""
	}.to_string()
}

fn get_graph() -> Vec<(Graph, Vec<json::Json>)> {
	// get json graph and node arguments from stage2.json
	let json_str_to_decode = File::open(&Path::new("./stage2.json")).read_to_string().unwrap();
	let json_object = json::from_str(json_str_to_decode.as_slice()).unwrap();
	let mut decoder = json::Decoder::new(json_object.clone());
	// extract enum-guarded arguments from json, do not deguard
	let args: Vec<Vec<json::Json>> = json_object.as_list().unwrap().iter().map(|x| x.search(&("nodes".to_string())).unwrap().as_list().unwrap().iter()
		.map(|x| {x.find(&("args".to_string())).unwrap().clone()}).collect()).collect();
	let y: Vec<Graph> = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
		Err(e) => fail!("json decoding error")
	};
	y.into_iter().zip(args.into_iter()).collect()
}


fn gen_function(g: Graph, args: Vec<json::Json>) -> ast::Item {
	let mut channel_stmts: Vec<ptr::P<ast::Stmt>> = vec!();
	let mut spawn_exprs: Vec<ptr::P<ast::Expr>> = vec!();
	let mut fnargv: Vec<ast::Arg> = vec!();
	let mut io: Vec<String> = vec!();
	// this does the work - iterate over nodes and arguments
	for node in g.nodes.clone().into_iter() {
		let mut rxers: Vec<String> = vec!();
		let mut txers: Vec<String> = vec!();
		let n = expand_prim(node.pname.clone());
		for &(ref e0, ref e1) in g.edges.iter() {
			if &node.uid == e0 {
				let mut ename = "tx".to_string();
				ename.push_str(e0.as_slice());
				ename.push_str(e1.as_slice());
				txers.push(ename);
			}
			else if &node.uid == e1 {
				let mut ename = "rx".to_string();
				ename.push_str(e0.as_slice());
				ename.push_str(e1.as_slice());
				rxers.push(ename);
			}
		}
		match n.as_slice() {
			"in" => {
				let mut x = "r".to_string();
				x.push_str(txers[0].as_slice().slice_from(1));
				io.push(x.clone());
				fnargv.push(abstrast::arg(x.as_slice(), "Receiver<T>"))
				}
			"out" => {
				let mut x = "t".to_string();
				x.push_str(rxers[0].as_slice().slice_from(1));
				fnargv.push(abstrast::arg(x.as_slice(), "Sender<T>"))
				}
			_ => {}
		}
	}
	for (node, arg) in g.nodes.clone().into_iter().zip(args.into_iter()) {
		let n = expand_prim(node.pname.clone());
		let mut rxers: Vec<String> = vec!();
		let mut txers: Vec<String> = vec!();
		let mut argv = vec!();
		for &(ref e0, ref e1) in g.edges.iter() {
			if &node.uid == e0 {
				let mut ename = "tx".to_string();
				ename.push_str(e0.as_slice());
				ename.push_str(e1.as_slice());
				txers.push(ename);
			}
			else if &node.uid == e1 {
				let mut ename = "rx".to_string();
				ename.push_str(e0.as_slice());
				ename.push_str(e1.as_slice());
				rxers.push(ename);
			}
		}
		match rxers.len() {
			0 => (),
			1 => argv.push(expr_path(rxers[0].as_slice())),
			_ => argv.push(expr_vec(rxers.iter().map(|x| expr_path(x.as_slice())).collect()))
		}
		match txers.len() {
			0 => (),
			1 => argv.push(expr_path(txers[0].as_slice())),
			_ => {
					let mut ftx = txers[0].as_slice().slice_to(13).to_string();
					ftx.push_str("0");
					let mut frx = ("r").to_string();
					frx.push_str(ftx.as_slice().slice_from(1));
					spawn_exprs.push(spawn("fork",
						vec!(expr_path(frx.as_slice()), expr_vec(txers.iter().map(
							|x| expr_path(x.as_slice())).collect()))));
						channel_stmts.push(stmt_let(pat_tuple(vec!(pat_name(ftx.as_slice()),
							pat_name(frx.as_slice()))), expr_call(expr_path("channel"), vec!())));
					argv.push(expr_path(ftx.as_slice()))
				}
			};

		argv.extend(
			match json_to_ast (arg.clone()) {
				Some(lits) => {
					match lits {
						ast::ExprVec(v) => if v.len() > 0 {v} else {
						match get_default_args(node.pname.clone()).as_slice() {
							"" => vec!(),
							x => vec!(parse_expr(x.to_string()))
						}},
						ast::ExprPath(_) /*| ast::ExprVstore(_, _)*/ => vec!(expr(lits)),
						_ => fail!("json to ast transcoding error"),
					}
				}
				None => vec!()
			}.into_iter());
		match n.as_slice() {
			"in" => {},
			"out" => {},
			_ => {
					spawn_exprs.push(spawn(n.as_slice(), argv));
					txers.iter().map(|txer| {
						let mut dstrm = ("r").to_string();
						dstrm.push_str(txer.as_slice().slice_from(1));
						if io.clone().iter().filter(|x| x.as_slice() == txer.as_slice()).count() < 1 {
							channel_stmts.push(stmt_let(pat_tuple(vec!(pat_name(txer.as_slice()),
								pat_name(dstrm.as_slice()))), expr_call(expr_path("channel"), vec!())));
							}
						}).last();
			}
		}
	};
	channel_stmts.extend(spawn_exprs.into_iter().map(|x| stmt_semi(x)));
	match g.args {
		Some(ref aargs) => {aargs.iter().map(|x| fnargv.push(abstrast::arg(x.as_slice(), "T"))).last();},
		None => ()
	};
	fn_item(g.name.as_slice(), fnargv, ty_nil(), block(channel_stmts, None))
}

fn main () {
	let forest: Vec<(Graph, Vec<json::Json>)> = get_graph();
	for &(ref g, _) in forest.iter() {
		let mut fname = g.name.clone();
		fname.push_str(".dot");
		let mut out = File::create(&Path::new(fname)).unwrap();
		graphviz::render(g, &mut out).unwrap();
	}
	let mut stage3 = File::create(&Path::new("stage3.rs")).unwrap();
	let boilerplate = File::open(&Path::new("./boilerplate.rs")).read_to_end().unwrap();
	stage3.write(boilerplate.as_slice()).unwrap();
	for z in forest.into_iter().map(|(x,y)| gen_function(x,y)) {
		 stage3.write(syntax::print::pprust::item_to_string(&z).as_bytes()).unwrap();
	}
}
