use robusta_jni::bridge;

use log::{info, LevelFilter};
use once_cell::sync::Lazy;
use rdkafka::config::{FromClientConfig, FromClientConfigAndContext};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{DeliveryResult, ProducerContext, ThreadedProducer};
use rdkafka::{ClientConfig, ClientContext};
use robusta_jni::jni::objects::AutoArray;
use robusta_jni::jni::sys::jbyte;
use simple_logger::SimpleLogger;

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

static INIT_LOGGING: Lazy<()> = Lazy::new(|| {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap()
});

#[bridge]
mod jni {
    use crate::{borrow_as_slice, LoggingProducerContext, LoggingThreadedProducer, INIT_LOGGING};
    use log::{error, info};
    use rdkafka::config::RDKafkaLogLevel;
    use rdkafka::producer::{BaseRecord, Producer};
    use rdkafka::ClientConfig;
    use robusta_jni::convert::{
        Field, IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue,
    };

    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::objects::{AutoLocal, JObject, JString, ReleaseMode};
    use robusta_jni::jni::JNIEnv;

    use robusta_jni::jni::sys::jlong;
    use std::error::Error;
    use std::ops::Deref;
    use std::time::Duration;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(org.apache.kafka.clients.producer)]
    pub struct DefaultRustKafkaProducer<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        #[field]
        producer: Field<'env, 'borrow, jlong>,
    }

    impl<'env: 'borrow, 'borrow> DefaultRustKafkaProducer<'env, 'borrow> {
        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}

        pub extern "jni" fn init(
            mut self,
            env: &'borrow JNIEnv<'env>,
            config: JObject<'env>,
        ) -> JniResult<()> {
            let _ = INIT_LOGGING.deref();

            let mut client_config = ClientConfig::new();

            client_config.extend(env.get_map(config)?.iter()?.map(|(key, value)| {
                (
                    String::from(env.get_string(JString::from(key)).unwrap()),
                    String::from(env.get_string(JString::from(value)).unwrap()),
                )
            }));

            let producer: LoggingThreadedProducer<LoggingProducerContext> = client_config
                .set_log_level(RDKafkaLogLevel::Debug)
                .create()
                .expect("Producer creation failed");

            self.producer
                .set(Box::into_raw(Box::new(producer)) as jlong)?;

            info!("Created producer from config {client_config:?}");

            Ok(())
        }

        pub extern "jni" fn topics(self) -> JniResult<Vec<String>> {
            let result = self
                .producer()?
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
            let jbyte_array =
                env.get_byte_array_elements(payload.into_inner(), ReleaseMode::NoCopyBack)?;

            let result = self.producer()?.0.send(
                BaseRecord::with_opaque_to(&topic, ())
                    .key(&key)
                    .payload(borrow_as_slice(&jbyte_array)),
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

        fn producer(&self) -> JniResult<&LoggingThreadedProducer<LoggingProducerContext>> {
            let producer =
                self.producer.get()? as *const LoggingThreadedProducer<LoggingProducerContext>;

            Ok(unsafe { &*producer as &LoggingThreadedProducer<LoggingProducerContext> })
        }
    }
}

fn borrow_as_slice<'a>(jbyte_array: &'a AutoArray<jbyte>) -> &'a [u8] {
    let data = jbyte_array.as_ptr() as *const u8;
    let len = jbyte_array.size().unwrap() as usize;
    unsafe { std::slice::from_raw_parts(data, len) }
}
