#![macro_use]

macro_rules! allow {
    ( $($slot:expr),* $(,)? ) => (Gate::new_allow(vec!($($slot),*)));
}

macro_rules! block {
    ( $($slot:expr),* $(,)? ) => (Gate::new_block(vec!($($slot),*)));
}

macro_rules! splitset {
    ( $($split:expr),* $(,)? ) => (SplitSet::new(btreeset!($($split),*)));
}

macro_rules! flow {
    ( $($flow_item:expr),* $(,)? ) => (Flow::new(vec!($($flow_item),*)));
}

pub mod gate;
pub mod walk;
pub mod flow;
