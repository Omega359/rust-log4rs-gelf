// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

use appender::BufferAppenderBuilder;
use gelf_logger::Value;
use log::Level;
use log4rs::append::Append;
use log4rs::config::{Deserialize, Deserializers};
use std::collections::BTreeMap;
use std::time::Duration;

struct BufferAppenderDeserializer;

impl Deserialize for BufferAppenderDeserializer {
    type Trait = dyn Append;
    type Config = Config;

    fn deserialize(
        &self,
        config: Config,
        _deserializers: &Deserializers,
    ) -> Result<Box<dyn Append>, anyhow::Error> {
        let appender = BufferAppenderBuilder::default()
            .set_level(config.level.clone())
            .set_hostname(config.hostname.clone().as_str())
            .set_port(config.port.clone())
            .set_null_character(config.null_character.clone())
            .set_buffer_size(config.buffer_size.clone())
            .extend_additional_field(config.additional_fields.clone())
            .set_connect_timeout(config.connect_timeout.map_or(None,|v| Some(Duration::from_secs(v)) ))
            .set_write_timeout(config.write_timeout.map_or(None,|v| Some(Duration::from_secs(v)) ));

        #[cfg(feature = "tls")]
        let appender = match true {
            _ => appender.set_use_tls(config.use_tls.clone())
        };

        Ok(Box::new(appender.build()?))
    }
}

pub fn deserializers() -> Deserializers {
    let mut d = Deserializers::default();
    d.insert("buffer", BufferAppenderDeserializer);
    d
}

/// Struct to manipulate configuration.
#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct Config {
    level: Level,
    hostname: String,
    port: u16,
    null_character: bool,
    buffer_size: Option<usize>,
    additional_fields: BTreeMap<String, Value>,
    connect_timeout: Option<u64>,
    write_timeout: Option<u64>,
    #[cfg(feature = "tls")]
    use_tls: bool,
}