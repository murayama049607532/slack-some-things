use std::env;

use anyhow::Context;
use dotenvy::dotenv;

pub mod create_table;
pub mod dist;
pub mod dist_target_map;
pub mod fetch_user_folder;
pub mod user_folder;
pub mod utils;

pub fn db_url() -> anyhow::Result<String> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").context("db_url is missing.")?;

    Ok(db_url)
}
