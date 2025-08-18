mod single;
mod batch;
mod debug;

pub use single::run_single_test;
pub use batch::{run_batch_tests, run_category_tests};
pub use debug::debug_test;