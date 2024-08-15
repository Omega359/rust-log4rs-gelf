// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;
use anyhow::Context;
use gelf_logger::{Batch, BatchProcessor, Config, init_processor};
use log4rs::append::Append;
use log::Record;
use serde_gelf::{GelfLevel, GelfRecord};
use serde_value::Value;

/// Struct to handle the GELF buffer.
///
/// ## Example
///
/// ```rust
/// extern crate serde_gelf;
/// extern crate serde_value;
///
/// use serde_gelf::GelfLevel;
/// use serde_value::Value;
///
/// fn main() {
///     use std::time::Duration;
/// let appender = log4rs_gelf::BufferAppender::builder()
///         .set_level(GelfLevel::Informational)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .set_buffer_duration(Some(Duration::from_millis(500)))
///         .put_additional_field("component", Value::String("rust-cs".to_string()))
///         .build()
///         .expect("Failed to create appender");
/// }
/// ```
pub struct BufferAppender {
    processor: BatchProcessor
}

/// Builder for [`BufferAppender`](struct.BufferAppender.html).
///
/// ## Example
///
/// ```rust
/// extern crate serde_gelf;
/// extern crate serde_value;
///
/// use serde_gelf::GelfLevel;
/// use serde_value::Value;
///
/// fn main() {
///     use std::time::Duration;
/// let builder = log4rs_gelf::BufferAppenderBuilder::default()
///         .set_level(GelfLevel::Informational)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .set_buffer_duration(Some(Duration::from_millis(500)))
///         .put_additional_field("component", Value::String("rust-cs".to_string()))
///         ;
/// }
/// ```
#[derive(Debug)]
pub struct BufferAppenderBuilder {
    level: GelfLevel,
    hostname: String,
    port: u64,
    use_tls: bool,
    async_buffer_size: Option<usize>,
    null_character: bool,
    buffer_size: Option<usize>,
    buffer_duration: Option<Duration>,
    additional_fields: BTreeMap<Value, Value>,
    // full_buffer_policy: Option<FullBufferPolicy>,
    connect_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl Default for BufferAppenderBuilder {
    fn default() -> BufferAppenderBuilder {
        BufferAppenderBuilder {
            level: GelfLevel::default(),
            hostname: "127.0.0.1".to_string(),
            port: 12202,
            use_tls: true,
            async_buffer_size: None,
            null_character: true,
            buffer_size: Some(100),
            buffer_duration: Some(Duration::from_millis(500)),
            additional_fields: {
                let mut additional_fields = BTreeMap::new();
                additional_fields.insert(Value::String("pkg_name".into()), Value::String(env!("CARGO_PKG_NAME").into()));
                additional_fields.insert(Value::String("pkg_version".into()), Value::String(env!("CARGO_PKG_VERSION").into()));
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
    pub fn set_level(mut self, level: GelfLevel) -> BufferAppenderBuilder {
        self.level = level;
        self
    }
    /// Sets the hostname of the remote server.
    pub fn set_hostname(mut self, hostname: &str) -> BufferAppenderBuilder {
        self.hostname = hostname.to_string();
        self
    }
    /// Sets the port of the remote server.
    pub fn set_port(mut self, port: u64) -> BufferAppenderBuilder {
        self.port = port;
        self
    }
    /// Activate transport security.
    pub fn set_use_tls(mut self, use_tls: bool) -> BufferAppenderBuilder {
        self.use_tls = use_tls;
        self
    }
    /// Sets the async buffer size
    pub fn set_async_buffer_size(mut self, async_buffer_size: Option<usize>) -> BufferAppenderBuilder {
        self.async_buffer_size = async_buffer_size;
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
    /// Sets the maximum lifetime (in milliseconds) of the buffer before send it to the remote
    /// server.
    pub fn set_buffer_duration(mut self, buffer_duration: Option<Duration>) -> BufferAppenderBuilder {
        self.buffer_duration = buffer_duration;
        self
    }
    /// Adds an additional data which will be appended to each log entry.
    pub fn put_additional_field(mut self, key: &str, value: Value) -> BufferAppenderBuilder {
        self.additional_fields.insert(Value::String(key.to_string()), value);
        self
    }
    /// Adds multiple additional data which will be appended to each log entry.
    pub fn extend_additional_field(mut self, additional_fields: BTreeMap<Value, Value>) -> BufferAppenderBuilder {
        self.additional_fields.extend(additional_fields);
        self
    }
    /// set the full buffer policy
    // pub fn set_full_buffer_policy(mut self, full_buffer_policy: Option<FullBufferPolicy>) -> BufferAppenderBuilder {
    //     self.full_buffer_policy = full_buffer_policy;
    //     self
    // }
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
        let mut builder = Config::builder()
            .set_level(self.level)
            .set_hostname(self.hostname)
            .set_port(self.port)
            .set_use_tls(self.use_tls)
            .set_null_character(self.null_character)
            .set_buffer_size(self.buffer_size.unwrap_or(100))
            .set_buffer_duration(self.buffer_duration.unwrap_or(Duration::from_millis(500)))
            .extend_additional_fields(self.additional_fields)
            .set_connect_timeout(self.connect_timeout)
            .set_write_timeout(self.write_timeout);

        if let Some(async_buffer_size) = self.async_buffer_size {
            builder = builder.set_async_buffer_size(async_buffer_size);
        }
        // if let Some(full_buffer_policy) = self.full_buffer_policy {
        //     builder = builder.set_full_buffer_policy(full_buffer_policy);
        // }

        let cfg = builder.build();

        Ok(BufferAppender {
            processor: init_processor(&cfg)?
        })
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
        self.processor.send(&GelfRecord::from(record)).context("")
    }
    fn flush(&self) {}
}