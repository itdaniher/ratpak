#![feature(globs)]

extern crate serialize;
extern crate syntax;
extern crate abstrast;
extern crate graphviz;

use graphviz::maybe_owned_vec::IntoMaybeOwnedVector;
use std::strbuf::StrBuf;
use std::io::File;
use std::path::Path;
use abstrast::*;
use syntax::ast;
use serialize::{json, Encodable, Decodable};
use std::str;

#[deriving(Decodable,Encodable,Clone)]
struct Node {
	pname: StrBuf,
	uid: StrBuf,
	label: StrBuf
}

type Edge = (StrBuf, StrBuf);

// naive graph format
#[deriving(Decodable,Encodable,Clone)]
struct Graph {
	edges: Vec<Edge>,
	nodes: Vec<Node>,
	name: StrBuf,
	args: Option<Vec<StrBuf>>,
	inrx: bool,
	outtx: bool,
}

impl<'a> graphviz::Labeller<'a, Node, Edge> for Graph {
    fn graph_id(&'a self) -> graphviz::Id<'a> { graphviz::Id::new(self.name.as_slice()) }
    fn node_id(&'a self, n: &Node) -> graphviz::Id<'a> {
        graphviz::Id::new(n.uid.clone().into_maybe_owned())
    }
    fn node_label<'a>(&'a self, n: &Node) -> graphviz::LabelText<'a> {
        graphviz::LabelStr(n.pname.clone().into_maybe_owned())
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

fn expandPrim(nodepname: StrBuf) -> StrBuf {
	let mut snp = nodepname.clone();
	let c0 = snp.shift_char().unwrap();
	let mut out: StrBuf = "".to_strbuf();
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
		'?' => {out = expandPrim(snp).append("Optional"); out.as_slice()},
		'/' => {out = expandPrim(snp).append("Across"); out.as_slice()},
		',' => {out = expandPrim(snp).append("Vecs"); out.as_slice()},
		 _  => nodepname.as_slice()
	}.to_strbuf()
}

fn getDefaultArgs(nodepname: StrBuf) -> StrBuf {
	let mut snp = nodepname.clone();
	let c0 = snp.shift_char().unwrap();
	let mut out: StrBuf = "".to_strbuf();
	match nodepname.len() {
		1..3 => match c0 {
			'*' => "1.0f32",
			'+' | 'Z' => "0.0f32",
			'/' => { out = getDefaultArgs(snp); out.as_slice()}
			',' => { match getDefaultArgs(snp).as_slice() {
					"" => "",
					x => {out = "range(0,512).map(|_| ".to_strbuf().append(x).append(").collect()"); out.as_slice()}
				}
			}
			_ => "",
		},
		_ => ""
	}.to_strbuf()
}

fn getGraph() -> Vec<(Graph, Vec<json::Json>)> {
	// get json graph and node arguments from stage2.json
	let json_str_to_decode = File::open(&Path::new("./stage2.json")).read_to_str().unwrap();
	let json_object = json::from_str(json_str_to_decode).unwrap();
	let mut decoder = json::Decoder::new(json_object.clone());
	// extract enum-guarded arguments from json, do not deguard
	let args: Vec<Vec<json::Json>> = json_object.as_list().unwrap().iter().map(|x| x.search(&("nodes".to_strbuf())).unwrap().as_list().unwrap().iter()
		.map(|x| {x.find(&("args".to_strbuf())).unwrap().clone()}).collect()).collect();
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
	let mut io: Vec<StrBuf> = vec!();
	// this does the work - iterate over nodes and arguments
	for node in g.nodes.clone().move_iter() {
		let mut rxers: Vec<StrBuf> = vec!();
		let mut txers: Vec<StrBuf> = vec!();
		let n = expandPrim(node.pname.clone());
		for &(ref e0, ref e1) in g.edges.iter() {
			if &node.uid == e0 {
				txers.push("tx".to_strbuf().append(e0.as_slice()).append(e1.as_slice()));
			}
			else if &node.uid == e1 {
				rxers.push("rx".to_strbuf().append(e0.as_slice()).append(e1.as_slice()));
			}
		}
		match n.as_slice() {
			"in" => {
				let x = ("r").to_strbuf().append(txers.get(0).as_slice().slice_from(1));
				io.push(x.clone());
				fnargv.push(abstrast::arg(x.as_slice(), "Receiver<f32>"))
				}
			"out" => {
				let x = ("t").to_strbuf().append(rxers.get(0).as_slice().slice_from(1));
				io.push(x.clone());
				fnargv.push(abstrast::arg(x.as_slice(), "Sender<f32>"))
				}
			_ => {}
		}
	}
	for (node, arg) in g.nodes.clone().move_iter().zip(args.move_iter()) {
		let mut rxers: Vec<StrBuf> = vec!();
		let mut txers: Vec<StrBuf> = vec!();
		for &(ref e0, ref e1) in g.edges.iter() {
			if &node.uid == e0 {
				let ename = "tx".to_strbuf().append(e0.as_slice()).append(e1.as_slice());
				txers.push(ename);
			}
			else if &node.uid == e1 {
				rxers.push("rx".to_strbuf().append(e0.as_slice()).append(e1.as_slice()));
			}
		}
		let n = expandPrim(node.pname.clone());
		let mut argv = vec!();
		match rxers.len() {
			0 => (),
			1 => argv.push(expr_path(rxers.get(0).as_slice())),
			_ => argv.push(expr_owned_vec(rxers.iter().map(|x| expr_path(x.as_slice())).collect()))
		}
		match txers.len() {
			0 => (),
			1 => argv.push(expr_path(txers.get(0).as_slice())),
			_ => {
					let ftx = txers.get(0).as_slice().slice_to(13).to_strbuf().append("0");
					let frx = ("r").to_strbuf().append(ftx.as_slice().slice_from(1));
					spawnExprs.push(spawn("fork",
						vec!(expr_path(frx.as_slice()), expr_owned_vec(txers.iter().map(
							|x| expr_path(x.as_slice())).collect()))));
						channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(ftx.as_slice()),
							pat_name(frx.as_slice()))), expr_call(expr_path("channel"), vec!())));
					argv.push(expr_path(ftx.as_slice()))
				}
			};

		argv.push_all_move(
			match JSONtoAST(arg.clone()) {
				Some(lits) => {
					match lits {
						ast::ExprVec(v) => if v.len() > 0 {v} else {
						match getDefaultArgs(node.pname.clone()).as_slice() {
							"" => vec!(),
							x => vec!(parse_expr(x.to_strbuf()))
						}},
						ast::ExprPath(_) | ast::ExprVstore(_, _) => vec!(expr(lits)),
						_ => fail!("{:?}", lits)
					}
				}
				None => vec!()
			});
		match n.as_slice() {
			"in" => {},
			"out" => {},
			_ => {
					spawnExprs.push(spawn(n.as_slice(), argv));
					txers.iter().map(|txer| {
						let dstrm = ("r").to_strbuf().append(txer.as_slice().slice_from(1));
						if io.clone().iter().filter(|x| x.as_slice() == txer.as_slice()).len() < 1 {
							channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(txer.as_slice()),
								pat_name(dstrm.as_slice()))), expr_call(expr_path("channel"), vec!())));
							}
						}).last();
			}
		}
	};
	channelStmts.push_all_move(spawnExprs.move_iter().map(|x| stmt_semi(x)).collect());
	match g.args {
		Some(ref aargs) => {aargs.iter().map(|x| fnargv.push(abstrast::arg(x.as_slice(), "f32"))).last();},
		None => ()
	};
	fn_item(g.name.as_slice(), fnargv, ty_nil(), block(channelStmts, None))
}

fn main () {
	let forest: Vec<(Graph, Vec<json::Json>)> = getGraph();
	let mut out = File::create(&Path::new("temps.dot")).unwrap();
	let (g, _) = forest.get(0).clone();
    graphviz::render(&g, &mut out);
	println!("{}", File::open(&Path::new("./boilerplate.rs")).read_to_str().unwrap());
	let o: Vec<StrBuf> = forest.move_iter().map(|(x,y)| genFunction(x,y)).map(|z| syntax::print::pprust::item_to_str(&z)).collect();
	for f in o.iter() {
		println!("{}", f);
	}
}
