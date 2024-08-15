// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

//! # log4rs_gelf
//!
//! [`log4rs`](https://docs.rs/log4rs/*/log4rs/index.html) is a highly configurable logging
//! framework modeled after Java's Logback and log4j libraries.
//!
//! The Graylog Extended Log Format ([GELF](http://docs.graylog.org/en/latest/pages/gelf.html#gelf-payload-specification))
//! is a log format that avoids the shortcomings of classic log formats. GELF is a great choice for
//! logging from within applications. You could use GELF to send every exception as a log message
//! to your Graylog cluster.
//!
//! This crate provides the GELF support in log4rs.
//!
//! ## Examples
//!
//! Configuration via a YAML file:
//!
//! ```yaml
//! appenders:
//!   ldp:
//!     kind: buffer
//!     additional_fields:
//!       component: rust-cs
//!     buffer_size: 5
//!     hostname: 127.0.0.1
//!     level: Info
//!     null_character: true
//!     port: 12202
//!     use_tls: false
//! root:
//!   appenders:
//!   - ldp
//!   level: info
//! ```
//!
//! ```rust,ignore
//! log4rs_gelf::init_file("/tmp/log4rs.yml", None).unwrap();
//! ```
//! Programmatically constructing a configuration:
//! ```rust
//! extern crate serde_gelf;
//! extern crate gelf_logger;
//! extern crate log4rs;
//! extern crate log;
//!
//! use serde_gelf::GelfLevel;
//! use gelf_logger::Value;
//! use log4rs::config::{Config, Appender, Root};
//! use log::{Level,LevelFilter};
//! use std::time::Duration;
//! fn main() {
//!    let buffer = log4rs_gelf::BufferAppender::builder()
//!        .set_level(Level::Info)
//!        .set_hostname("localhost")
//!        .set_port(12202)
//!        .set_use_tls(false)
//!        .set_null_character(true)
//!        .set_buffer_size(Some(5))
//!        .put_additional_field("component", Value::String("rust-cs".to_string()))
//!        .build()
//!        .unwrap();
//!
//!    let config = Config::builder()
//!        .appender(Appender::builder().build("gelf", Box::new(buffer)))
//!        .build(Root::builder().appender("gelf").build(LevelFilter::Info))
//!        .unwrap();
//!
//!    log4rs_gelf::init_config(config).unwrap();
//!
//!    // Do whatever
//!
//! }
//! ```
#![doc(
html_logo_url = "https://eu.api.ovh.com/images/com-square-bichro.png",
html_favicon_url = "https://www.ovh.com/favicon.ico",
)]
extern crate gelf_logger;
extern crate log;
extern crate log4rs;
extern crate serde_gelf;
extern crate serde_value;
extern crate anyhow;

use log::SetLoggerError;
pub use appender::{BufferAppender, BufferAppenderBuilder};

mod file;
mod appender;

/// Initializes the global logger as a log4rs logger configured via a file.
///
/// Configuration is read from a file located at the provided path on the
/// filesystem and components are created from the provided `Deserializers`.
///
/// Any nonfatal errors encountered when processing the configuration are
/// reported to stderr.
///
/// ### Warning
///
/// The logging system may only be initialized once.
///
/// ## Example
///
/// ```rust, ignore
/// extern crate log4rs_gelf;
///
/// fn main() {
///     log4rs_gelf::init_file("/tmp/log4rs.yml", None).unwrap();
///
///     // Do whatever
///
/// }
/// ```
///
pub fn init_file<P>(path: P, deserializers: Option<log4rs::config::Deserializers>) -> anyhow::Result<()> where P: AsRef<std::path::Path> {
    log4rs::init_file(path, deserializers.unwrap_or(file::deserializers()))
}

/// Initializes the global logger as a log4rs logger with the provided config.
///
/// A `Handle` object is returned which can be used to adjust the logging
/// configuration.
///
/// ### Warning
///
/// The logging system may only be initialized once.
///
/// ## Example
///
/// ```rust
/// extern crate serde_gelf;
/// extern crate gelf_logger;
/// extern crate log4rs;
/// extern crate log;
///
///use serde_gelf::GelfLevel;
///use gelf_logger::Value;
///use log4rs::config::{Config, Appender, Root};
///use std::time::Duration;
///use log4rs::append::Append;
///use log::{Level, LevelFilter};
///
/// fn main() {
///    let buffer = log4rs_gelf::BufferAppender::builder()
///        .set_level(Level::Info)
///        .set_hostname("localhost")
///        .set_port(12202)
///        .set_use_tls(false)
///        .set_null_character(true)
///        .set_buffer_size(Some(5))
///        .put_additional_field("component", Value::String("rust-cs".to_string()))
///        .build()
///        .unwrap();
///
///    let config = Config::builder()
///        .appender(Appender::builder().build("gelf", Box::new(buffer)))
///        .build(Root::builder().appender("gelf").build(LevelFilter::Info))
///        .unwrap();
///
///    log4rs_gelf::init_config(config).unwrap();
///
///    // Do whatever
/// }
/// ```
///
pub fn init_config(config: log4rs::config::Config) -> Result<log4rs::Handle, SetLoggerError> {
    Ok(log4rs::init_config(config)?)
}

