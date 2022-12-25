package com.example.demo

import org.apache.kafka.clients.producer.KafkaProducer
import org.apache.kafka.clients.producer.ProducerConfig
import org.apache.kafka.clients.producer.ProducerRecord
import org.apache.kafka.common.serialization.StringSerializer
import java.util.*

fun kafkaProducer() {
    val producer = KafkaProducer<String, String>(Properties().also {
        it[ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG] = StringSerializer::class.java
        it[ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG] = StringSerializer::class.java
        it[ProducerConfig.METADATA_MAX_AGE_CONFIG] = 15_000
        it[ProducerConfig.BOOTSTRAP_SERVERS_CONFIG] = "my-cluster-kafka-0.my-cluster-kafka-brokers.kafka.svc:9092"
    })

    println("Constructor calls")

    // 1. DNS resolution in constructor call thread (ClientUtils.parseAndValidateAddresses)
    // 2. DNS resolution in producer thread
    // 3. API_VERSIONS request in producer thread
    // 4. METADATA request in producer thread (all topics - empty list)
    // 5. INIT_PRODUCER_ID request in producer thread

    Thread.sleep(5_000)

    println("partitionsForTopic calls")
    println("partitions for my-topic: ${producer.partitionsFor("my-topic")}")

    // 1. DNS resolution in producer thread (check if still happens in constructor call too)
    // 2. API_VERSIONS request in producer thread
    // 3. METADATA request in producer thread (specified topic)

    Thread.sleep(5_000)

    println("send calls")

    producer.send(ProducerRecord("my-topic", "cevaaa"))

    // 1. PRODUCE request in producer thread
    // OR
    // 1. API_VERSIONS request in producer thread
    // 2. METADATA request in producer thread (specified topic)
    // 3. PRODUCE request in producer thread

    Thread.sleep(2_000)

    println("metadata refresh")

    // 1. METADATA request in producer thread (all seen topics so far)

    println("partitions for unknown: ${producer.partitionsFor("unknown")}")

    Thread.sleep(3600_000)
}
