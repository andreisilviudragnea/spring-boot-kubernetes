package org.apache.kafka.clients.producer

class RustKafkaProducer {
    external fun init(bootstrapServers: String?, useSsl: Boolean)

    external fun fetchMetadata(bootstrapServers: String?): ArrayList<String>

    external fun send(bootstrapServers: String?, topic: String?, key: String?, payload: String?)

    external fun close(bootstrapServers: String?)
}
