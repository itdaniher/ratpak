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
use std::vec;

pub fn asRe<T: Float> ( d: Vec<T> ) -> Vec<complex::Cmplx<T>> { d.iter().map(|&x| {complex::Cmplx {re: x, im: num::zero()}}).collect() }

pub fn applicator<T: Clone+Send>(u: Receiver<T>, v: Sender<T>, f: fn(T)->T) {
	loop {
		v.send(f(u.recv()))
	}
}

pub fn applicatorVecs<T: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, f: |&T|->T) {
	loop {
		v.send(u.recv().iter().map(|x|f(x)).collect())
	}
}

pub fn uintTof32Vecs(u: Receiver<Vec<uint>>, v: Sender<Vec<f32>>) {
	loop {
		v.send(u.recv().iter().map(|&x| x as f32).collect())
	}
}

pub fn uintTof32(u: Receiver<uint>, v: Sender<f32>) {
	loop {
		v.send(u.recv() as f32)
	}
}

pub fn vec<T: Clone>(u: &[T]) -> Vec<T> {
	vec::Vec::from_slice(u)
}

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

pub fn delayVecs<T: Send+Clone>(u: Receiver<T>, v: Sender<T>, c: T) {
	delay(u, v, c);
}

pub fn shaper<T: Send+Clone>(u: Receiver<Option<T>>, v: Sender<Vec<T>>, l: uint) {
	let mut x = vec!();
	loop {
		match u.recv() {
			Some(y) => x.push(y),
			None => x = vec!(),
		}
		if x.len() == l {
			v.send(x.clone());
			x = vec!();
		}
	}
}

pub fn binconv(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, l: ~[uint]) {
	loop {
		v.send(eat(u.recv().slice_from(0), l.clone()))
	}
}
