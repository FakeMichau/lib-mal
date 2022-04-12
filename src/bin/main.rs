#![allow(unused)]

use lib_mal::prelude::*;

use simple_log::console;
use simple_log::{info, warn};

fn main() {
    simple_log::console("debug");
    warn!("This is a library, this binary is for testing purposes");
}
