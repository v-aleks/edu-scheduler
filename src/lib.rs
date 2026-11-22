//! Offline-first планировщик расписания преподавателей.

pub mod app;
pub mod db;
pub mod domain;
pub mod ui;

pub use domain::model::{Slot, SlotKind, Teacher, Weekday};
