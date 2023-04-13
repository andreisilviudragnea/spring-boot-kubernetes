package org.apache.kafka.clients.producer

class RustKafkaProducer(bootstrapServers: String, useSsl: Boolean) {
    init {
        init(bootstrapServers, useSsl)
    }

    companion object {
        init {
            System.loadLibrary("producer")
        }
    }

    private external fun init(bootstrapServers: String, useSsl: Boolean)

    external fun fetchMetadata(bootstrapServers: String): ArrayList<String>

    external fun send(bootstrapServers: String, topic: String, key: String, payload: ByteArray)

    external fun close(bootstrapServers: String)
}
