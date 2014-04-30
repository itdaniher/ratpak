#![feature(globs)]

extern crate serialize;
extern crate syntax;
extern crate abstrast;

use std::io::File;
use std::path::Path;
use std::strbuf::StrBuf;
use abstrast::*;
use syntax::ast;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable,Encodable,Clone)]
struct Node {
	pname: ~str
}

#[deriving(Decodable,Encodable)]
struct Graph {
	edges: Vec<Vec<~str>>,
	nodes: Vec<(~str, Node)>,
	name: ~str,
	consts: Option<Vec<~str>>,
	inrx: bool,
	outtx: bool,
}

fn getGraph() -> (Graph, ~[json::Json]) {
	let json_str_to_decode = File::open(&Path::new("./stage2.json")).read_to_str().unwrap();
	let json_object = json::from_str(json_str_to_decode.to_owned()).unwrap();
	let mut decoder = json::Decoder::new(json_object.clone());
	let args: ~[json::Json] = json_object.search(&~"nodes").unwrap().as_list().unwrap().iter().map(|x| {x.as_list().unwrap()[1].find(&~"args").unwrap().clone()}).collect();
	let y: Graph = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
		Err(e) => fail!("Decoding error: {}", e)
	};
	println!("// nodes, edges: {}", (y.nodes.len(), y.edges.len()));
	(y, args)
}

fn main () {
	let (y, args) = getGraph();
	let mut channelStmts: Vec<ast::P<ast::Stmt>> = vec!();
	let mut spawnExprs: Vec<ast::P<ast::Expr>> = vec!();

	for ((uid, node), arg) in y.nodes.clone().move_iter().zip(args.move_iter()) {
		let mut rxers: Vec<~str> = vec!();
		let mut txers: Vec<~str> = vec!();
		for edge in y.edges.iter() {
			if &uid == edge.get(0) {
				let mut e = StrBuf::from_str("tx");
				e.push_str(edge.get(0).clone());
				e.push_str(edge.get(1).clone());
				txers.push(e.into_owned());
			}
			else if &uid == edge.get(1) {
				let mut e = StrBuf::from_str("rx");
				e.push_str(edge.get(0).clone());
				e.push_str(edge.get(1).clone());
				rxers.push(e.into_owned());
			}
			else {
			}
		}
		let nodepname = node.pname;

		let n = if nodepname.slice_from(0) == "*." { "mulAcrossVecs".to_str() }
			else if nodepname.slice_from(0) == "*" { "mulAcross".to_str() }
			else if nodepname.slice_from(0) == "+" {"sumAcross".to_str()}
			else if nodepname.slice_from(0) == "Z" {"delay".to_str()}
			else if nodepname.slice_from(0) == "%" {"grapes".to_str()}
			else if nodepname.slice_from(0) == "b" {"binconv".to_str()}
			else if nodepname.slice_from(0) == "$" {"shaper".to_str()}
			else if nodepname.slice_from(0) == "?" {"matcher".to_str()}
			else { nodepname.clone() };
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
					spawnExprs.push(spawn(expr_call(expr_path("fork".to_str()),
						vec!(expr_path(frx), expr_owned_vec(txers.iter().map(|x| expr_path(x.slice_from(0))).collect())))));
					channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(ftx.clone()), pat_name(frx.clone()))), expr_call(expr_path("channel"), vec!())));
					argv.push(expr_path(ftx))
				}
			};

		argv.push_all_move(
			match JSONtoAST(arg.clone()) {
				Some(lits) => {
					match lits {
						ast::ExprVec(v) => v,
						ast::ExprPath(v) => vec!(expr(ast::ExprPath(v))),
						ast::ExprVstore(v, _) => vec!(expr(lits)),
						_ => fail!("{:?}", lits)
				}}
				None => {
					if nodepname.slice_from(0) == "+" {
						vec!(parse_expr("0"))
					}
					else if nodepname.slice_from(0) == "*" {
						vec!(parse_expr("1"))
					}
					else {
						vec!()
					}
				}
			}
		);

		spawnExprs.push(spawn(expr_call(expr_path(n), argv)));

		txers.iter().map(|txer| {
			let dstrm = (~"r").append((*txer).slice_from(1));
			channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(*txer), pat_name(dstrm))), expr_call(expr_path("channel"), vec!())))
		}).last();
	}

	channelStmts.push_all_move(spawnExprs.move_iter().map(|x| stmt_semi(x)).collect());

	let function = match y.consts.clone() {
		Some(fnargs) => fn_item(y.name,
			fnargs.move_iter().map(|x| {ast::Arg {ty: ty_infer(), pat: pat_name(x.slice_from(0)), id: 0}}).collect(),
			ty_nil(), block(channelStmts, None)),
		None => fn_item(y.name, vec!(), ty_nil(), block(channelStmts, None))
	};

	println!("{}", File::open(&Path::new("./boilerplate.rs")).read_to_str().unwrap());
	println!("{}", syntax::print::pprust::item_to_str(&function));
}
