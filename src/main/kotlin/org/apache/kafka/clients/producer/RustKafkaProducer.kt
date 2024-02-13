package org.apache.kafka.clients.producer

interface RustKafkaProducer : AutoCloseable {
    fun topics(): List<String>

    fun send(
        topic: String,
        key: String,
        payload: ByteArray,
    )
}
