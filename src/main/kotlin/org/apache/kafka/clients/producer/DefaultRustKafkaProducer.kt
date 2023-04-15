package org.apache.kafka.clients.producer

class DefaultRustKafkaProducer(config: Map<String, String>) : RustKafkaProducer {
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

    external override fun topics(): List<String>

    external override fun send(topic: String, key: String, payload: ByteArray)

    external override fun close()
}
