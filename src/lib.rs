#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

///This Library Requires that Tower_Cookies is used as an active layer.
//#[cfg(feature = "postgres")]
//#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]

//#[cfg(feature = "mysql")]
//#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
//mod mysql;

//#[cfg(feature = "sqlite")]
//#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
//mod sqlite;
mod databases;
mod sessions;

pub use databases::*;
pub use sessions::{SessionBind, SqlxSessionConfig};
//#[cfg(feature = "postgres")]
//#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
//pub use postgres::{SqlxSession, SqlxSessionLayer};

//#[cfg(feature = "mysql")]
//#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
//pub use mysql::{MysqlSession, MysqlSessionLayer};

//#[cfg(feature = "sqlite")]
//#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
//pub use sqlite::{SqliteSession, SqliteSessionLayer};
