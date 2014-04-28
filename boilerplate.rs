#![feature(globs)];
extern crate kpn;
extern crate bitfount;
extern crate rtlsdr;
extern crate native;
extern crate vidsink2;

use rtlsdr::*;
use kpn::*;
use bitfount::*;
use vidsink2::*;

static localhost: &'static str = "localhost";

pub fn fork<T: Clone+Send>(u: std::comm::Receiver<T>, v: ~[std::comm::Sender<T>]) {
	loop {
		let x = u.recv();
		for y in v.iter() {
			y.send(x.clone());
		}
	}
}

pub fn mulAcross<T: Float+Send>(u: ~[std::comm::Receiver<T>], v: std::comm::Sender<T>, C: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(C, |B, A| B*A))
	}
}

pub fn sumAcross<T: Float+Send>(u: ~[std::comm::Receiver<T>], v: std::comm::Sender<T>, C: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(C, |B, A| B+A))
	}
}

pub fn grapes<T: Send>( u: ~[std::comm::Receiver<T>], v: std::comm::Sender<T>) {
	use std::comm::Select;
	let sel = Select::new();
	let mut hs: Vec<std::comm::Handle<T>> = vec!();
	for x in u.iter() {
		let mut h = sel.handle(x);
		unsafe {
			h.add();
		}
		hs.push(h);
	};
	let hids: ~[uint] = hs.iter().map(|x| x.id()).collect();
	loop {
		let y = sel.wait();
		v.send(u[hids.iter().enumerate().filter_map(|(a, &b)| if b == y { Some(a) } else { None }).next().unwrap()].recv());
	}
}
