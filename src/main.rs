#![feature(entry_or_default)]
#![feature(macro_at_most_once_rep)]
#![feature(type_ascription)]

#[macro_use] extern crate maplit;
#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate nom;
extern crate regex;

mod parallel;
mod token;
mod parser;

fn main() {}
