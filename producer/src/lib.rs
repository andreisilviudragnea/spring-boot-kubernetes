use robusta_jni::bridge;

use std::sync::RwLock;

use log::{info, LevelFilter};
use once_cell::sync::Lazy;
use rdkafka::config::{FromClientConfig, FromClientConfigAndContext};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{DeliveryResult, ProducerContext, ThreadedProducer};
use rdkafka::{ClientConfig, ClientContext};
use robusta_jni::jni::objects::AutoArray;
use robusta_jni::jni::sys::{jbyte, jsize};
use simple_logger::SimpleLogger;
use std::collections::HashMap;

pub struct LoggingThreadedProducer<C: ProducerContext + 'static>(ThreadedProducer<C>);

pub struct LoggingProducerContext;

impl ClientContext for LoggingProducerContext {}

impl ProducerContext for LoggingProducerContext {
    type DeliveryOpaque = ();

    fn delivery(
        &self,
        delivery_result: &DeliveryResult<'_>,
        _delivery_opaque: Self::DeliveryOpaque,
    ) {
        info!("Delivery result {delivery_result:?}")
    }
}

impl FromClientConfig for LoggingThreadedProducer<LoggingProducerContext> {
    fn from_config(config: &ClientConfig) -> KafkaResult<Self> {
        ThreadedProducer::from_config_and_context(config, LoggingProducerContext)
            .map(LoggingThreadedProducer)
    }
}

static PRODUCERS_MAP: Lazy<
    RwLock<HashMap<String, LoggingThreadedProducer<LoggingProducerContext>>>,
> = Lazy::new(|| {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
    RwLock::new(HashMap::new())
});

#[bridge]
mod jni {
    use crate::{borrow_as_slice, LoggingProducerContext, LoggingThreadedProducer, PRODUCERS_MAP};
    use log::{error, info};
    use rdkafka::config::RDKafkaLogLevel;
    use rdkafka::producer::{BaseRecord, Producer};
    use rdkafka::ClientConfig;
    use robusta_jni::convert::{
        Field, IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue,
    };

    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::objects::{AutoLocal, JObject, ReleaseMode};
    use robusta_jni::jni::JNIEnv;

    use robusta_jni::jni::sys::jlong;
    use std::error::Error;
    use std::time::Duration;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(org.apache.kafka.clients.producer)]
    pub struct RustKafkaProducer<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        #[field]
        producer: Field<'env, 'borrow, jlong>,
    }

    impl<'env: 'borrow, 'borrow> RustKafkaProducer<'env, 'borrow> {
        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}

        pub extern "jni" fn init(
            mut self,
            bootstrap_servers: String,
            use_ssl: bool,
        ) -> JniResult<()> {
            let mut client_config = ClientConfig::new();

            // client_config.set("compression.type", "lz4");

            info!("bootstrap.servers {}", bootstrap_servers);
            client_config.set("bootstrap.servers", bootstrap_servers.clone());
            client_config.set("broker.address.family", "v4");
            if use_ssl {
                client_config.set("security.protocol", "SSL");
            }

            client_config.set_log_level(RDKafkaLogLevel::Debug);

            let producer: LoggingThreadedProducer<LoggingProducerContext> =
                client_config.create().expect("Producer creation failed");

            self.producer
                .set(Box::into_raw(Box::new(producer)) as jlong)?;

            let _ = PRODUCERS_MAP.read();

            info!("Created producer {bootstrap_servers} {use_ssl}");

            Ok(())
        }

        pub extern "jni" fn fetchMetadata(self) -> JniResult<Vec<String>> {
            let producer =
                self.producer.get()? as *const LoggingThreadedProducer<LoggingProducerContext>;

            let producer =
                unsafe { &*producer as &LoggingThreadedProducer<LoggingProducerContext> };

            let result = producer
                .0
                .client()
                .fetch_metadata(None, Duration::from_secs(5));

            let metadata = match result {
                Ok(metadata) => metadata,
                Err(error) => {
                    error!("Error on fetching metadata: {error:?}");
                    return Ok(vec![]);
                }
            };

            let topics = metadata
                .topics()
                .iter()
                .map(|topic| topic.name().to_string())
                .collect();

            info!("Topics {topics:?}");

            Ok(topics)
        }

        pub extern "jni" fn send(
            self,
            env: &'borrow JNIEnv<'env>,
            topic: String,
            key: String,
            payload: JObject<'env>,
        ) -> JniResult<()> {
            let producer =
                self.producer.get()? as *const LoggingThreadedProducer<LoggingProducerContext>;

            let producer =
                unsafe { &*producer as &LoggingThreadedProducer<LoggingProducerContext> };

            let jobject = payload.into_inner();

            let jbyte_array = env.get_byte_array_elements(jobject, ReleaseMode::NoCopyBack)?;
            let length = env.get_array_length(jobject)?;

            let result = producer.0.send(
                BaseRecord::with_opaque_to(&topic, ())
                    .key(&key)
                    .payload(borrow_as_slice(&jbyte_array, length)),
            );

            info!("Send result {result:?}");

            Ok(())
        }

        pub extern "jni" fn close(self) -> JniResult<()> {
            let producer =
                self.producer.get()? as *mut LoggingThreadedProducer<LoggingProducerContext>;

            unsafe { Box::from_raw(producer) };

            info!("Closed producer");

            Ok(())
        }
    }
}

fn borrow_as_slice<'a>(jbyte_array: &'a AutoArray<jbyte>, length: jsize) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(jbyte_array.as_ptr() as *const u8, length as usize) }
}
