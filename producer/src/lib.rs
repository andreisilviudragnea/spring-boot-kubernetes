use robusta_jni::bridge;

use std::sync::RwLock;

use once_cell::sync::Lazy;
use rdkafka::producer::{DefaultProducerContext, ThreadedProducer};
use std::collections::HashMap;

static GLOBAL_DATA: Lazy<RwLock<HashMap<String, ThreadedProducer<DefaultProducerContext>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[bridge]
mod jni {
    use crate::GLOBAL_DATA;
    use log::{info, LevelFilter};
    use rdkafka::config::RDKafkaLogLevel;
    use rdkafka::producer::BaseRecord;
    use rdkafka::ClientConfig;
    use robusta_jni::convert::{
        Field, IntoJavaValue, Signature, TryFromJavaValue, TryIntoJavaValue,
    };
    use robusta_jni::jni::errors::Error as JniError;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::JNIEnv;
    use simple_logger::SimpleLogger;
    use std::error::Error;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(org.apache.kafka.clients.producer)]
    pub struct KafkaProducer<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
    }

    impl<'env: 'borrow, 'borrow> KafkaProducer<'env, 'borrow> {
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

            let mut map = GLOBAL_DATA.write().unwrap();

            map.insert(bootstrap_servers.clone(), producer);

            println!("Created producer {bootstrap_servers} {use_ssl}");

            Ok(())
        }

        pub extern "jni" fn send(
            self,
            bootstrap_servers: String,
            topic: String,
            key: String,
            payload: String,
        ) -> JniResult<()> {
            let map = GLOBAL_DATA.read().unwrap();

            let producer = map.get(&bootstrap_servers).unwrap();

            let result = producer.send(
                BaseRecord::with_opaque_to(&topic, ())
                    .key(&key)
                    .payload(&payload),
            );

            println!("Send result {result:?}");

            Ok(())
        }

        pub extern "jni" fn close(self, bootstrap_servers: String) -> JniResult<()> {
            let mut map = GLOBAL_DATA.write().unwrap();

            map.remove(&bootstrap_servers);

            println!("Closed producer {bootstrap_servers}");

            Ok(())
        }
    }
}
