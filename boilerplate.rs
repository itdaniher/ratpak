#![feature(globs)];
extern crate kpn;
extern crate collections;
extern crate bitfount;
extern crate rtlsdr;
extern crate native;
extern crate vidsink2;
extern crate kissfft;
extern crate num;
extern crate dsputils;
extern crate time;
extern crate oblw;
extern crate outlet;


use kissfft::fft;
use collections::bitv;
use rtlsdr::*;
use kpn::*;
use bitfount::*;
use vidsink2::*;
use outlet::*;
use native::task;
use num::complex;
use std::comm::{Receiver, Sender, channel, Messages};
use std::num;
use std::vec;

pub fn asRe<T: Float> ( d: Vec<T> ) -> Vec<complex::Cmplx<T>> { d.iter().map(|&x| {complex::Cmplx {re: x, im: num::zero()}}).collect() }

pub fn applicator<T: Clone+Send>(u: Receiver<T>, v: Sender<T>, f: |T|->T) {
	loop {
		v.send(f(u.recv()))
	}
}

pub fn softSource<T: Send+Clone>(v: Sender<T>, f: |x: Sender<T>|) {
	f(v.clone());
	let (s,r) = channel::<()>();
	r.recv();
}

pub fn matcher<T: Send+Clone, U: Send+Clone>(u: Receiver<T>, v: Sender<U>, f: |x: Messages<T>,v: Sender<U>|) {
	f(u.iter(), v)
}

pub fn crossApplicator<T: Clone+Send, U: Clone+Send>(u: Receiver<T>, v: Sender<U>, f: |T|->U) {
	loop {
		v.send(f(u.recv()))
	}
}

pub fn crossApplicatorVecs<T: Clone+Send, U: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<U>>, f: |&T|->U) {
	loop {
		v.send(u.recv().iter().map(|x|f(x)).collect())
	}
}

pub fn applicatorVecs<T: Clone+Send>(u: Receiver<Vec<T>>, v: Sender<Vec<T>>, f: |&T|->T) {
	loop {
		v.send(u.recv().iter().map(|x|f(x)).collect())
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

pub fn mulAcross<T: Float+Send>(u: Receiver<T>, v: Sender<T>, c: T) {
	loop {
		v.send(u.recv()*c)
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
	let mut timer = std::io::Timer::new().unwrap();
	loop {
		for x in u.iter() {
			match x.try_recv() {
				Ok(d) => v.send(d),
				Err(_) => ()
			}
			timer.sleep(10);
		}
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
			None if x.len() == l => {v.send(x.clone()); x = vec!();},
			None => {x = vec!();},
		}
	}
}

pub fn binconv(u: Receiver<Vec<uint>>, v: Sender<Vec<uint>>, l: ~[uint]) {
	loop {
		v.send(eat(u.recv().slice_from(0), l.clone()))
	}
}

