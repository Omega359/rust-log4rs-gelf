// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

use std::{fmt, thread};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};

use gelf_logger::{Batch, BatchProcessor, Buffer, Config, Event, GelfTcpOutput, Metronome};
use log4rs::append::Append;
use log::Record;
use serde_gelf::{GelfLevel, GelfRecord};
use serde_value::Value;

/// Struct to handle the GELF buffer.
///
/// ## Example
///
/// ```rust
/// use serde_gelf::GelfLevel;
/// use serde_value::Value;
///
/// fn main() {
///     let appender = log4rs_gelf::BufferAppender::builder()
///         .set_level(GelfLevel::Informational)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .set_buffer_duration(Some(5))
///         .put_additional_field("component", Value::String("rust-cs".to_string()))
///         .build()
///         .expect("Failed to create appender")
///         ;
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
/// use serde_gelf::GelfLevel;
/// use serde_value::Value;
///
/// fn main() {
///     let builder = log4rs_gelf::BufferAppenderBuilder::default()
///         .set_level(GelfLevel::Informational)
///         .set_hostname("localhost")
///         .set_port(12202)
///         .set_use_tls(false)
///         .set_null_character(true)
///         .set_buffer_size(Some(5))
///         .set_buffer_duration(Some(5))
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
    null_character: bool,
    buffer_size: Option<usize>,
    buffer_duration: Option<u64>,
    additional_fields: BTreeMap<Value, Value>,
}

impl Default for BufferAppenderBuilder {
    fn default() -> BufferAppenderBuilder {
        BufferAppenderBuilder {
            level: GelfLevel::default(),
            hostname: "127.0.0.1".to_string(),
            port: 12202,
            use_tls: true,
            null_character: true,
            buffer_size: Some(100),
            buffer_duration: Some(500),
            additional_fields: {
                let mut additional_fields = BTreeMap::new();
                additional_fields.insert(Value::String("pkg_name".into()), Value::String(env!("CARGO_PKG_NAME").into()));
                additional_fields.insert(Value::String("pkg_version".into()), Value::String(env!("CARGO_PKG_VERSION").into()));
                additional_fields
            },
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
    /// Sets the maximum lifetime (in milli seconds) of the buffer before send it to the remote
    /// server.
    pub fn set_buffer_duration(mut self, buffer_duration: Option<u64>) -> BufferAppenderBuilder {
        self.buffer_duration = buffer_duration;
        self
    }
    /// Adds an additional data which will be append to each log entry.
    pub fn put_additional_field(mut self, key: &str, value: Value) -> BufferAppenderBuilder {
        self.additional_fields.insert(Value::String(key.to_string()), value);
        self
    }
    /// Adds multiple additional data which will be append to each log entry.
    pub fn extend_additional_field(mut self, additional_fields: BTreeMap<Value, Value>) -> BufferAppenderBuilder {
        self.additional_fields.extend(additional_fields);
        self
    }
    /// Invoke the builder and return a [`BufferAppender`](struct.BufferAppender.html).
    pub fn build(self) -> Result<BufferAppender, gelf_logger::Error> {
        let cfg = Config::builder()
            .set_level(self.level)
            .set_hostname(self.hostname)
            .set_port(self.port)
            .set_use_tls(self.use_tls)
            .set_null_character(self.null_character)
            .set_buffer_size(self.buffer_size.unwrap_or(100))
            .set_buffer_duration(self.buffer_duration.unwrap_or(500))
            .extend_additional_fields(self.additional_fields)
            .build();

        let (tx, rx): (SyncSender<Event>, Receiver<Event>) = sync_channel(10_000_000);

        if let &Some(duration) = cfg.buffer_duration() {
            let ctx = tx.clone();
            Metronome::start(duration, ctx);
        }

        let gelf_level = cfg.level().clone();
        let arx = Arc::new(Mutex::new(rx));
        thread::spawn(move || {
            let _ = Buffer::new(arx, GelfTcpOutput::from(&cfg)).run();
        });

        Ok(BufferAppender {
            processor: BatchProcessor::new(tx, gelf_level)
        })
    }
}


impl BufferAppender {
    /// Creates a new [`BufferAppenderBuilder`](struct.BufferAppenderBuilder.html).
    #[cfg(not(feature = "ovh-ldp"))]
    pub fn builder() -> BufferAppenderBuilder {
        BufferAppenderBuilder::default()
    }
    /// Creates a new [`BufferAppenderBuilder`](struct.BufferAppenderBuilder.html) preconfigured for OVH [LDP](https://docs.ovh.com/gb/en/logs-data-platform/).
    #[cfg(feature = "ovh-ldp")]
    pub fn builder(hostname: &str, token: &str) -> BufferAppenderBuilder {
        BufferAppenderBuilder::default()
            .set_hostname(hostname)
            .set_level(GelfLevel::Informational)
            .put_additional_field("X-OVH-TOKEN", serde_value::Value::String(token.to_string()))
    }
}

impl fmt::Debug for BufferAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GelfAppender").finish()
    }
}


impl Append for BufferAppender {
    fn append(&self, record: &Record) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        match self.processor.send(&GelfRecord::from(record)) {
            Ok(()) => Ok(()),
            Err(exc) => Err(Box::new(exc))
        }
    }
    fn flush(&self) {}
}