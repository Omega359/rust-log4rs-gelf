// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

use anyhow::Context;
use gelf_logger::Value;
use gelf_logger::{Builder, GelfLogger};
use log::{Level, Log, Record};
use log4rs::append::Append;
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

/// Struct to handle the GELF buffer.
///
/// ## Example
///
/// ```rust
/// extern crate log;
/// extern crate serde_gelf;
/// extern crate gelf_logger;
///
/// use log::Level;
/// use serde_gelf::GelfLevel;
/// use gelf_logger::Value;
/// use std::time::Duration;
///
/// fn main() {
///     let buffer = log4rs_gelf::BufferAppender::builder()
///         .set_level(Level::Info)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .put_additional_field("component", Value::String("rust-cs".to_string()))
///         .build()
///         .expect("Failed to create appender");
/// }
/// ```
pub struct BufferAppender {
    gelf_logger: GelfLogger
}

/// Builder for [`BufferAppender`](struct.BufferAppender.html).
///
/// ## Example
///
/// ```rust
/// extern crate log;
/// extern crate serde_gelf;
/// extern crate gelf_logger;
///
/// use log::Level;
/// use serde_gelf::GelfLevel;
/// use gelf_logger::Value;
/// use std::time::Duration;
///
/// fn main() {
/// let builder = log4rs_gelf::BufferAppenderBuilder::default()
///         .set_level(Level::Info)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .put_additional_field("component", Value::String("rust-cs".to_string()));
/// }
/// ```
#[derive(Debug)]
pub struct BufferAppenderBuilder {
    level: Level,
    hostname: String,
    port: u16,
    #[cfg(feature = "tls")]
    use_tls: bool,
    null_character: bool,
    buffer_size: Option<usize>,
    additional_fields: BTreeMap<String, Value>,
    connect_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl Default for BufferAppenderBuilder {
    fn default() -> BufferAppenderBuilder {
        BufferAppenderBuilder {
            level: Level::Info,
            hostname: "127.0.0.1".to_string(),
            port: 12202,
            #[cfg(feature = "tls")]
            use_tls: true,
            null_character: true,
            buffer_size: Some(100),
            additional_fields: {
                let mut additional_fields = BTreeMap::new();
                additional_fields.insert("pkg_name".into(), Value::String(env!("CARGO_PKG_NAME").into()));
                additional_fields.insert("pkg_version".into(), Value::String(env!("CARGO_PKG_VERSION").into()));
                additional_fields
            },
            connect_timeout: None,
            write_timeout: None,
        }
    }
}


impl BufferAppenderBuilder {
    /// Sets threshold for this logger to level. Logging messages which are less severe than level
    /// will be ignored.
    pub fn set_level(mut self, level: Level) -> BufferAppenderBuilder {
        self.level = level;
        self
    }
    /// Sets the hostname of the remote server.
    pub fn set_hostname(mut self, hostname: &str) -> BufferAppenderBuilder {
        self.hostname = hostname.to_string();
        self
    }
    /// Sets the port of the remote server.
    pub fn set_port(mut self, port: u16) -> BufferAppenderBuilder {
        self.port = port;
        self
    }
    /// Activate transport security.
    #[cfg(feature = "tls")]
    pub fn set_use_tls(mut self, use_tls: bool) -> BufferAppenderBuilder {
        self.use_tls = use_tls;
        self
    }
    /// Adds a NUL byte (`\0`) after each entry.
    pub fn set_null_character(mut self, null_character: bool) -> BufferAppenderBuilder {
        self.null_character = null_character;
        self
    }
    /// Sets the upperbound limit on the number of records that can be placed in the buffer, once
    /// this size has been reached, the buffer will be sent to the remote server.
    pub fn set_buffer_size(mut self, buffer_size: Option<usize>) -> BufferAppenderBuilder {
        self.buffer_size = buffer_size;
        self
    }
    /// Adds an additional data which will be appended to each log entry.
    pub fn put_additional_field(mut self, key: &str, value: Value) -> BufferAppenderBuilder {
        self.additional_fields.insert(key.to_string(), value);
        self
    }
    /// Adds multiple additional data which will be appended to each log entry.
    pub fn extend_additional_field(mut self, additional_fields: BTreeMap<String, Value>) -> BufferAppenderBuilder {
        self.additional_fields.extend(additional_fields);
        self
    }
    /// set the connection timeout
    pub fn set_connect_timeout(mut self, connect_timeout: Option<Duration>) -> BufferAppenderBuilder {
        self.connect_timeout = connect_timeout;
        self
    }
    /// set the write timeout
    pub fn set_write_timeout(mut self, write_timeout: Option<Duration>) -> BufferAppenderBuilder {
        self.write_timeout = write_timeout;
        self
    }
    /// Invoke the builder and return a [`BufferAppender`](struct.BufferAppender.html).
    pub fn build(self) -> Result<BufferAppender, gelf_logger::Error> {
        let builder = Builder::new()
            .filter_level(self.level.to_level_filter())
            .hostname(self.hostname)
            .port(self.port)
            .null_character(self.null_character)
            .buffer_size(self.buffer_size.unwrap_or(100))
            .extend_additional_fields(self.additional_fields)
            .connect_timeout(self.connect_timeout)
            .write_timeout(self.write_timeout)
            .background_error_handler(Some(|err| {
                eprintln!("{err:?}");
            }));

        #[cfg(feature = "tls")]
        let builder = match true {
            _ => builder.tls(self.use_tls)
        };

        Ok(BufferAppender { gelf_logger: builder.build()? })
    }
}


impl BufferAppender {
    /// Creates a new [`BufferAppenderBuilder`](struct.BufferAppenderBuilder.html).
    pub fn builder() -> BufferAppenderBuilder {
        BufferAppenderBuilder::default()
    }
}

impl fmt::Debug for BufferAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GelfAppender").finish()
    }
}


impl Append for BufferAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        self.gelf_logger.append(record).context("")
    }
    fn flush(&self) {
        Log::flush(&self.gelf_logger)
    }
}