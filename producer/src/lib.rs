use robusta_jni::bridge;

use std::sync::RwLock;

use log::info;
use once_cell::sync::Lazy;
use rdkafka::config::{FromClientConfig, FromClientConfigAndContext};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{DeliveryResult, ProducerContext, ThreadedProducer};
use rdkafka::{ClientConfig, ClientContext};
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
        ThreadedProducer::from_config_and_context(config, LoggingProducerContext).map(LoggingThreadedProducer)
    }
}

static PRODUCERS_MAP: Lazy<RwLock<HashMap<String, LoggingThreadedProducer<LoggingProducerContext>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[bridge]
mod jni {
    use crate::PRODUCERS_MAP;
    use log::{info, LevelFilter};
    use rdkafka::config::RDKafkaLogLevel;
    use rdkafka::producer::BaseRecord;
    use rdkafka::ClientConfig;
    use robusta_jni::convert::{IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue};

    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::JNIEnv;
    use simple_logger::SimpleLogger;
    use std::error::Error;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(org.apache.kafka.clients.producer)]
    pub struct RustKafkaProducer<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
    }

    impl<'env: 'borrow, 'borrow> RustKafkaProducer<'env, 'borrow> {
        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}

        pub extern "jni" fn init(self, bootstrap_servers: String, use_ssl: bool) -> JniResult<()> {
            SimpleLogger::new()
                .with_level(LevelFilter::Info)
                .init()
                .unwrap();

            let mut client_config = ClientConfig::new();

            // client_config.set("compression.type", "lz4");

            info!("bootstrap.servers {}", bootstrap_servers);
            client_config.set("bootstrap.servers", bootstrap_servers.clone());
            if use_ssl {
                client_config.set("security.protocol", "SSL");
            }

            client_config.set_log_level(RDKafkaLogLevel::Debug);

            let producer = client_config.create().expect("Producer creation failed");

            let mut map = PRODUCERS_MAP.write().unwrap();

            map.insert(bootstrap_servers.clone(), producer);

            info!("Created producer {bootstrap_servers} {use_ssl}");

            Ok(())
        }

        pub extern "jni" fn send(
            self,
            bootstrap_servers: String,
            topic: String,
            key: String,
            payload: String,
        ) -> JniResult<()> {
            let map = PRODUCERS_MAP.read().unwrap();

            let producer = map.get(&bootstrap_servers).unwrap();

            let result = producer.0.send(
                BaseRecord::with_opaque_to(&topic, ())
                    .key(&key)
                    .payload(&payload),
            );

            info!("Send result {result:?}");

            Ok(())
        }

        pub extern "jni" fn close(self, bootstrap_servers: String) -> JniResult<()> {
            let mut map = PRODUCERS_MAP.write().unwrap();

            map.remove(&bootstrap_servers);

            info!("Closed producer {bootstrap_servers}");

            Ok(())
        }
    }
}
