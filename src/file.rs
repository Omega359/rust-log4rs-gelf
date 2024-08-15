// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The log4rs-gelf Authors. All rights reserved.

use log4rs::append::Append;
use log4rs::config::{Deserialize, Deserializers};

use appender::BufferAppenderBuilder;

struct BufferAppenderDeserializer;

impl Deserialize for BufferAppenderDeserializer {
    type Trait = dyn Append;
    type Config = gelf_logger::Config;

    fn deserialize(
        &self,
        config: gelf_logger::Config,
        _deserializers: &Deserializers,
    ) -> Result<Box<dyn Append>, anyhow::Error> {
        let appender = BufferAppenderBuilder::default()
            .set_level(config.level().clone())
            .set_hostname(config.hostname())
            .set_port(config.port().clone())
            .set_use_tls(config.use_tls().clone())
            .set_async_buffer_size(config.async_buffer_size())
            .set_null_character(config.null_character().clone())
            .set_buffer_size(config.buffer_size().clone())
            .set_buffer_duration(config.buffer_duration().clone())
            .extend_additional_field(config.additional_fields().clone())
            .set_connect_timeout(config.connect_timeout())
            .set_write_timeout(config.write_timeout());

        Ok(Box::new(appender.build()?))
    }
}

pub fn deserializers() -> Deserializers {
    let mut d = Deserializers::default();
    d.insert("buffer", BufferAppenderDeserializer);
    d
}