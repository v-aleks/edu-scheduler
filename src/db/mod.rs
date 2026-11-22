//! Локальное хранилище на SQLite.

mod migrations;
mod repo;

pub use repo::{Database, DbError};
