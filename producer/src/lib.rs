use robusta_jni::bridge;
use std::sync::Once;
use std::time::Instant;

use log::{info, LevelFilter};
use rdkafka::config::{FromClientConfig, FromClientConfigAndContext};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{DeliveryResult, ProducerContext, ThreadedProducer};
use rdkafka::{ClientConfig, ClientContext};
use robusta_jni::jni::objects::AutoArray;
use robusta_jni::jni::sys::jbyte;
use simple_logger::SimpleLogger;

pub struct LoggingThreadedProducer(ThreadedProducer<LoggingProducerContext>);

pub struct LoggingProducerContext;

impl ClientContext for LoggingProducerContext {}

impl ProducerContext for LoggingProducerContext {
    type DeliveryOpaque = Box<ProducerDeliveryOpaque>;

    fn delivery(
        &self,
        delivery_result: &DeliveryResult<'_>,
        delivery_opaque: Self::DeliveryOpaque,
    ) {
        let elapsed = delivery_opaque.start.elapsed().as_millis();
        info!("Delivery result: {delivery_result:?}; Elapsed: {elapsed} ms")
    }
}

impl FromClientConfig for LoggingThreadedProducer {
    fn from_config(config: &ClientConfig) -> KafkaResult<Self> {
        ThreadedProducer::from_config_and_context(config, LoggingProducerContext)
            .map(LoggingThreadedProducer)
    }
}

#[derive(Debug)]
pub struct ProducerDeliveryOpaque {
    start: Instant,
}

static ONCE: Once = Once::new();

fn init() {
    ONCE.call_once(|| {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .with_threads(true)
            .init()
            .unwrap()
    });
}

#[bridge]
mod jni {
    use crate::{borrow_as_slice, init, LoggingThreadedProducer, ProducerDeliveryOpaque};
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

    use std::time::{Duration, Instant};

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
            init();

            let mut client_config = ClientConfig::new();

            for (key, value) in env.get_map(config)?.iter()? {
                client_config.set(
                    String::from(env.get_string(JString::from(key))?), // TODO: Handle missing key gracefully
                    String::from(env.get_string(JString::from(value))?), // TODO: Handle missing value gracefully
                );
            }

            let producer: LoggingThreadedProducer = client_config
                .set_log_level(RDKafkaLogLevel::Debug)
                .create()
                .expect("Producer creation failed");

            self.producer
                .set(Box::into_raw(Box::new(producer)) as jlong)?;

            info!("Created producer from config {client_config:?}");

            Ok(())
        }

        pub extern "jni" fn topics(self, env: &'borrow JNIEnv<'env>) -> JniResult<Vec<String>> {
            let result = self
                .producer()?
                .0
                .client()
                .fetch_metadata(None, Duration::from_secs(5));

            let metadata = match result {
                Ok(metadata) => metadata,
                Err(error) => {
                    error!("Error on fetching metadata: {error:?}");

                    self.drop_producer()?;

                    return env
                        .throw_new("java/lang/IllegalStateException", format!("{error:?}"))
                        .map(|_| vec![]);
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
                BaseRecord::with_opaque_to(
                    &topic,
                    Box::new(ProducerDeliveryOpaque {
                        start: Instant::now(),
                    }),
                )
                .key(&key)
                .payload(borrow_as_slice(&jbyte_array)),
            );

            info!("Send result {result:?}");

            Ok(())
        }

        pub extern "jni" fn close(self) -> JniResult<()> {
            self.drop_producer()?;

            info!("Closed producer");

            Ok(())
        }

        fn producer(&self) -> JniResult<&LoggingThreadedProducer> {
            let producer = self.producer.get()? as *const LoggingThreadedProducer;

            Ok(unsafe { &*producer as &LoggingThreadedProducer })
        }

        fn drop_producer(&self) -> JniResult<()> {
            let producer = self.producer.get()? as *mut LoggingThreadedProducer;

            unsafe { Box::from_raw(producer) };

            Ok(())
        }
    }
}

fn borrow_as_slice<'a>(jbyte_array: &'a AutoArray<jbyte>) -> &'a [u8] {
    let data = jbyte_array.as_ptr() as *const u8;
    let len = jbyte_array.size().unwrap() as usize;
    unsafe { std::slice::from_raw_parts(data, len) }
}
