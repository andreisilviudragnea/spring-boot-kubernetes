package org.apache.kafka.clients.producer

class RustKafkaProducer(config: Map<String, String>) : AutoCloseable {
    private val producer: Long external get

    init {
        init(config)
    }

    companion object {
        init {
            System.loadLibrary("producer")
        }
    }

    private external fun init(config: Map<String, String>)

    external fun fetchMetadata(): List<String>

    external fun send(topic: String, key: String, payload: ByteArray)

    external override fun close()
}
