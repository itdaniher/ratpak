extern crate syntax;
extern crate serialize;
use std::default::Default;

use serialize::json;
use syntax::ast;
use syntax::ast::P;
use syntax::ast_util;
use syntax::codemap;
use syntax::parse::token;
use syntax::abi;
use syntax::parse::token::intern_and_get_ident;
use syntax::owned_slice::OwnedSlice;
use std::string::String;

pub fn path(name: &str, ty: Option<ast::Ty_>) -> ast::Path {
	ast::Path {
		span: codemap::DUMMY_SP,
		global: false,
		segments: vec!(
			ast::PathSegment {
				identifier: token::str_to_ident(name),
				lifetimes: Default::default(),
				// TODO: some way to change the type allowing this constructor to be used for fndef
				types: match ty { Some(n) => OwnedSlice::from_vec(vec!(P(ast::Ty { id: 0, node: n, span: codemap::DUMMY_SP }))), None => Default::default()}
			}
		),
	}
}

pub fn arg(e: &str, n: &str) -> ast::Arg {
	ast::Arg { ty: ty_path(n),
		pat: pat_name(e),
		id: 0
	}
}

pub fn fn_item(name: &str, inputs: Vec<ast::Arg>, output: ast::P<ast::Ty>, block: ast::P<ast::Block>) -> ast::Item{
	let generics = match inputs.len() {
		0 => ast_util::empty_generics(),
		_ => ast::Generics {
			lifetimes: Vec::new(),
			ty_params: OwnedSlice::from_vec(vec!(
				ast::TyParam {
					ident: token::str_to_ident("T"),
					id: 0,
					sized: ast::StaticSize,
					bounds: OwnedSlice::from_vec(vec!(
						ast::TraitTyParamBound( ast::TraitRef {
							path: path("core::num::Float", None),
							ref_id: 0
						}),
						ast::TraitTyParamBound( ast::TraitRef {
							path: path("core::kinds::Send", None),
							ref_id: 0
						}),
						)),
					default: None,
					span: codemap::DUMMY_SP
				}))}
	};
	let decl = ast::FnDecl {
		inputs: inputs,
		output: output,
		cf: ast::Return,
		variadic: false
	};
	ast::Item {
		ident: token::str_to_ident(name),
		attrs: vec!(),
		id: 0,
		node: ast::ItemFn(ast::P(decl), ast::NormalFn, abi::Rust, generics, block),
		vis: ast::Public,
		span: codemap::DUMMY_SP,
	}
}

pub fn spawn(fname: &str, args: Vec<P<ast::Expr>>) -> P<ast::Expr> {
	let exp: P<ast::Expr> = expr_call(expr_path(fname.clone()), args);
	let decl = ast::FnDecl {
		inputs: vec!(),
		output: ty_infer(),
		cf: ast::Return,
		variadic: false
	};
	expr_call(parse_expr(format!("task::TaskBuilder::named(\"{name}\").spawn", name=fname)), vec!(expr(ast::ExprProc(P(decl), block(vec!(), Some(expr(ast::ExprBlock(block(vec!(),Some(exp))))))))))
}

pub fn block(stmts: Vec<P<ast::Stmt>>, expr: Option<P<ast::Expr>>) -> P<ast::Block> {
	P(ast::Block {
		view_items: vec!(),
		stmts: stmts,
		expr: expr,
		id: 0,
		rules: ast::DefaultBlock,
		span: codemap::DUMMY_SP,
	})
}

pub fn expr(node: ast::Expr_) -> P<ast::Expr> {
	P(ast::Expr {
		id: 0,
		node: node,
		span: codemap::DUMMY_SP,
	})
}

pub fn expr_lit(lit: ast::Lit_) -> P<ast::Expr> {
	expr(ast::ExprLit(P(codemap::dummy_spanned(lit))))
}

pub fn expr_str(s: &str) -> P<ast::Expr> {
	expr_lit(ast::LitStr(intern_and_get_ident(s), ast::CookedStr))
}

pub fn expr_owned_vec(l: Vec<P<ast::Expr>>) -> P<ast::Expr> {
	expr(ast::ExprVec(l))
}

pub fn expr_char(c: char) -> P<ast::Expr> {
	expr_lit(ast::LitChar(c))
}

pub fn expr_path(p: &str) -> P<ast::Expr> {
	expr(ast::ExprPath(path(p, None)))
}

pub fn expr_tuple(l: Vec<P<ast::Expr>>) -> P<ast::Expr> {
	expr(ast::ExprTup(l))
}

pub fn expr_vec(l: Vec<P<ast::Expr>>) -> P<ast::Expr> {
	expr(ast::ExprVec(l))
}

pub fn expr_call(f: P<ast::Expr>, args: Vec<P<ast::Expr>>) -> P<ast::Expr> {
	expr(ast::ExprCall(f, args))
}

pub fn pat(p: ast::Pat_) -> P<ast::Pat> {
	P(ast::Pat {
		id: 0,
		node: p,
		span: codemap::DUMMY_SP,
	})
}

pub fn pat_name(name: &str) -> P<ast::Pat> {
	pat(ast::PatIdent(ast::BindByValue(ast::MutImmutable), codemap::dummy_spanned(syntax::ast::Ident::new(syntax::parse::token::intern(name))), None))
}

pub fn pat_tuple(items: Vec<P<ast::Pat>>) -> P<ast::Pat> {
	pat(ast::PatTup(items))
}

pub fn pat_wild() -> P<ast::Pat> {
	pat(ast::PatWild)
}

pub fn pat_wild_multi() -> P<ast::Pat> {
	pat(ast::PatWildMulti)
}

pub fn ty_infer() -> P<ast::Ty> {
	P(ast::Ty {
		id: 0,
		node: ast::TyInfer,
		span: codemap::DUMMY_SP,
	})
}

pub fn ty_nil() -> P<ast::Ty> {
	P(ast::Ty {
		id: 0,
		node: ast::TyNil,
		span: codemap::DUMMY_SP
	})
}

pub fn ty_path(e: &str) -> P<ast::Ty> {
	P(ast::Ty {
		id: 0,
		node: ast::TyPath(path(e, None),None,0),
		span: codemap::DUMMY_SP
	})
}

pub fn stmt_let(pat: P<ast::Pat>, expr: P<ast::Expr>) -> P<ast::Stmt> {
	P(codemap::dummy_spanned(ast::StmtDecl(
		P(codemap::dummy_spanned(
			ast::DeclLocal(P(ast::Local {
				ty: ty_infer(),
				pat: pat,
				init: Some(expr),
				id: 0,
				span: codemap::DUMMY_SP,
				source: ast::LocalLet
			}))
		)),
		0
	)))
}
pub fn stmt_semi(expr: P<ast::Expr>) -> P<ast::Stmt> {
	P(codemap::dummy_spanned(ast::StmtSemi(expr, 0)))
}


pub fn parse_expr(e: String) -> P<ast::Expr> {
	let ps = syntax::parse::new_parse_sess();
	let mut p = syntax::parse::new_parser_from_source_str(&ps, vec!(), String::from_str("file"), e);
	let r = p.parse_expr();
	p.abort_if_errors();
	r
}

pub fn parse_stmt(e: String) -> P<ast::Stmt> {
	let ps = syntax::parse::new_parse_sess();
	let mut p = syntax::parse::new_parser_from_source_str(&ps, vec!(), String::from_str("file"), e);
	let r = p.parse_stmt(vec!());
	p.abort_if_errors();
	r
}

pub fn JSONtoAST(jsonobj: json::Json) -> Option<ast::Expr_> {
	match jsonobj {
		json::Number(v) if (v - (v as int) as f64).abs() < 10.0*Float::epsilon() => Some(ast::ExprLit(P(codemap::dummy_spanned(ast::LitIntUnsuffixed(v as i64))))),
		json::Number(v) => Some(ast::ExprLit(P(codemap::dummy_spanned(ast::LitFloatUnsuffixed(syntax::parse::token::intern_and_get_ident(format!("{}", v).as_slice())))))),
		json::String(v) => Some(ast::ExprPath(path(v.as_slice(), None))),
		json::List(l) => if l.len() == 1 && l.get(0).is_list() == true {
			Some(ast::ExprVstore(expr_vec((l.get(0).as_list().unwrap()).iter().filter_map(|a| {JSONtoAST(a.clone())}).map(|a| expr(a)).collect()), ast::ExprVstoreUniq))}
		else {
			Some(ast::ExprVec(l.move_iter().filter_map(|a| {JSONtoAST(a)}).map(|a| expr(a)).collect()))},
		json::Boolean(v) => Some(ast::ExprLit(P(codemap::dummy_spanned(ast::LitBool(v))))),
		json::Null => None,
		_ => None
	}
}
