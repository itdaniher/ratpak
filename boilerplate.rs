#![feature(globs)];
extern crate kpn;
extern crate bitfount;
extern crate rtlsdr;
extern crate native;

use rtlsdr::*;
use kpn::*;
use bitfount::*;

static localhost: &str = "localhost";

