#![feature(globs)];
extern crate kpn;
extern crate bitfount;
extern crate rtlsdr;
extern crate native;
extern crate vidsink2;
extern crate kissfft;
extern crate num;
extern crate dsputils;

use native::task;
use num::complex;
use kissfft::FFT;
use rtlsdr::*;
use kpn::*;
use bitfount::*;
use vidsink2::*;
use std::comm::{Receiver, Sender, Select, Handle, channel};
use std::num;

static localhost: &'static str = "localhost";

pub fn asRe<T: Float> ( d: Vec<T> ) -> Vec<complex::Cmplx<T>> { d.iter().map(|&x| {complex::Cmplx {re: x, im: num::zero()}}).collect() }

pub fn fork<T: Clone+Send>(u: Receiver<T>, v: ~[Sender<T>]) {
	loop {
		let x = u.recv();
		for y in v.iter() {
			y.send(x.clone());
		}
	}
}

pub fn mulAcross<T: Float+Send>(u: ~[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c, |b, a| b*a))
	}
}

pub fn mulAcrossVecs<T: Float+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.recv().iter().zip(c.iter()).map(|(&x, &y)| x*y).collect())
	}
}

pub fn sumAcross<T: Float+Send>(u: ~[Receiver<T>], v: Sender<T>, c: T) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c, |b, a| b+a))
	}
}

pub fn sumAcrossVecs<T: Float+Send>(u: ~[Receiver<Vec<T>>], v: Sender<Vec<T>>, c: Vec<T>) {
	loop {
		v.send(u.iter().map(|y| y.recv()).fold(c.clone(), |b, a| a.iter().zip(b.iter()).map(|(&d, &e)| d+e).collect()))
	}
}

pub fn grapes<T: Send>(u: ~[Receiver<T>], v: Sender<T>) {
	let sel = Select::new();
	let mut hs: Vec<Handle<T>> = vec!();
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

pub fn delay<T: Send+Clone>(u: Receiver<T>, v: Sender<T>, c: T) {
	v.send(c);
	loop {
		v.send(u.recv());
	}
}

pub fn shaper<T: Send+Clone>(u: Receiver<T>, v: Sender<Vec<T>>, l: uint) {
	loop {
		v.send(range(0, l).map(|_| u.recv()).collect())
	}
}

pub fn binconv(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, l: ~[uint]) {
	loop {
		v.send(eat(u.recv().slice_from(0), l.clone()))
	}
}
