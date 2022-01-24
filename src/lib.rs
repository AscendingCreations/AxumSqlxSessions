#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod databases;
pub mod sessions;

pub use databases::{SqlxDatabaseConnection, SqlxDatabasePool};
pub use sessions::{SessionError, SqlxSession, SqlxSessionConfig, SqlxSessionLayer};
