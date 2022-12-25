package com.example.demo

import org.apache.kafka.clients.producer.KafkaProducer
import org.apache.kafka.clients.producer.ProducerConfig
import org.apache.kafka.common.serialization.StringSerializer
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication
import java.util.*

@SpringBootApplication
class DemoApplication

fun main(args: Array<String>) {
	val producer = KafkaProducer<String, String>(Properties().also {
		it[ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG] = StringSerializer::class.java
		it[ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG] = StringSerializer::class.java
		it[ProducerConfig.BOOTSTRAP_SERVERS_CONFIG] = "my-cluster-kafka-0.my-cluster-kafka-brokers.kafka.svc:9092"
	})

	Thread.sleep(5_000)

	println(producer.partitionsFor("my-topic"))

	Thread.sleep(60_000)

	runApplication<DemoApplication>(*args)
}
