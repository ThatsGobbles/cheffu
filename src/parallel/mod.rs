#![macro_use]

macro_rules! splitset {
    ( $($split:expr),* $(,)? ) => (SplitSet::new(btreeset!($($split),*)));
}

macro_rules! flow {
    ( $($flow_item:expr),* $(,)? ) => (Flow::new(vec!($($flow_item),*)));
}

pub mod gate;
pub mod walk;
pub mod flow;
