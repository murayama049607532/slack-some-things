pub mod create_table;
pub mod dist;
pub mod dist_target_map;
pub mod fetch_user_folder;
pub mod user_folder;
pub mod utils;

pub const DB_URL: &str = "sqlite://sqlite.db";
const DB_TEST_URL: &str = "sqlite://sqlite_test.db";
