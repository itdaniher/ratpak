#![feature(globs)]

extern crate serialize;
extern crate syntax;
extern crate ast;

use std::io::File;
use std::path::Path;
use std::strbuf::StrBuf;
use ast::*;
use serialize::{json, Encodable, Decodable};


#[deriving(Decodable,Encodable, Clone)]
struct Node {
	pname: ~str,
	oct: uint,
	ict: uint
}

#[deriving(Decodable,Encodable)]
struct Graph {
	edges: ~[~[~str]],
	nodes: ~[(~str, Node)]
}

fn main () {
	let json_str_to_decode = File::open(&Path::new("./temps.json")).read_to_str().unwrap();
	let json_object = json::from_str(json_str_to_decode.to_owned()).unwrap();
	let mut decoder = json::Decoder::new(json_object.clone());
	let args: ~[&json::Json] = json_object.search(&~"nodes").unwrap().as_list().unwrap().iter().map(|x| {x.as_list().unwrap()[1].find(&~"args").unwrap()}).collect();
	let y: Graph = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
		Err(e) => fail!("Decoding error: {}", e)
	};
	assert!(y.nodes.len() > 0)
	assert!(y.edges.len() > 0)
	let mut channelStmts: Vec<syntax::ast::P<syntax::ast::Stmt>> = vec!();
	let mut spawnExprs: Vec<syntax::ast::P<syntax::ast::Expr>> = vec!();
	for ((uid, node), arg) in y.nodes.clone().move_iter().zip(args.move_iter()) {
		let mut rxers: ~[~str] = ~[];
		let mut txers: ~[~str] = ~[];
		for edge in y.edges.iter() {
			if uid == edge[0] {
				let mut e = StrBuf::from_str("tx");
				e.push_str(edge[0]);
				e.push_str(edge[1]);
				txers.push(e.into_owned());
			}
			else if uid == edge[1] {
				let mut e = StrBuf::from_str("rx");
				e.push_str(edge[0]);
				e.push_str(edge[1]);
				rxers.push(e.into_owned());
			}
			else {
			}
		}
		let mut argv: Vec<syntax::ast::P<syntax::ast::Expr>> = match (rxers.len(), txers.len()) {
			(0, 0) => vec!(),
			(_, 0) => vec!(expr_path(rxers[0].slice_from(0))),
			(0, _) => vec!(expr_path(txers[0].slice_from(0))),
			(_, _) => vec!(expr_path(rxers[0].slice_from(0)), expr_path(txers[0].slice_from(0)))
		};
		argv.push_all_move(match JSONtoAST(arg.clone()) {
			Some(lits) => {
				match lits {
				syntax::ast::ExprVec(v) => v,
				_ => fail!("{:?}", lits)
			}}
			None => vec!()
		});
		spawnExprs.push(spawn(expr_call(expr_path(node.pname),argv)));
		txers.move_iter().map(|txer| {
			let dstrm = (~"r").append(txer.slice_from(1));
			channelStmts.push(stmt_let(pat_tuple(vec!(pat_name(txer), pat_name(dstrm))), expr_call(expr_path("std::comm::channel"), vec!())))
		}).last();
	}
	channelStmts.push_all_move(spawnExprs.move_iter().map(|x| stmt_semi(x)).collect());
	let main = fn_item("main", vec!(), ty_nil(), block(channelStmts, None));
	println!("{}", include_str!("boilerplate.rs"));
	println!("{}", syntax::print::pprust::item_to_str(&main));
}
