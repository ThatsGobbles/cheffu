#![macro_use]

macro_rules! splitset {
    ( $($split:expr),* $(,)? ) => (SplitSet::new(btreeset!($($split),*)));
}

macro_rules! flow {
    ( $($flow_item:expr),* $(,)? ) => (Flow::new(vec!($($flow_item),*)));
}

macro_rules! allow {
    ( $($slot:expr),* $(,)? ) => (Gate::new_allow(vec!($($slot),*)));
}

macro_rules! block {
    ( $($slot:expr),* $(,)? ) => (Gate::new_block(vec!($($slot),*)));
}

macro_rules! csplitset {
    ( $($split:expr),* $(,)? ) => (CowSplitSet::new(btreeset!($($split),*)));
}

macro_rules! cflow {
    ( $($flow_item:expr),* $(,)? ) => (CowFlow::new(vec!($($flow_item),*)));
}

pub mod gate;
// pub mod flow;
// pub mod split;
pub mod walk;
// pub mod scope;
pub mod cow_flow;
pub mod cow_split;
