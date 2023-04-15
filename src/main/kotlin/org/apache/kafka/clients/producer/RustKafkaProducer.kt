package org.apache.kafka.clients.producer

class RustKafkaProducer(bootstrapServers: String, useSsl: Boolean) : AutoCloseable {
    private val producer: Long external get

    init {
        init(bootstrapServers, useSsl)
    }

    companion object {
        init {
            System.loadLibrary("producer")
        }
    }

    private external fun init(bootstrapServers: String, useSsl: Boolean)

    external fun fetchMetadata(): ArrayList<String>

    external fun send(topic: String, key: String, payload: ByteArray)

    external override fun close()
}
