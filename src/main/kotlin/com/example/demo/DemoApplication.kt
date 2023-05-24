package com.example.demo

import io.micrometer.core.instrument.logging.LoggingMeterRegistry
import org.apache.kafka.clients.producer.DefaultRustKafkaProducer
import org.apache.kafka.clients.producer.RustKafkaProducer
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import org.springframework.boot.actuate.autoconfigure.metrics.KafkaMetricsAutoConfiguration
import org.springframework.boot.autoconfigure.AutoConfiguration
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.context.annotation.Bean
import org.springframework.kafka.core.KafkaTemplate

@SpringBootApplication
@AutoConfiguration(after = [KafkaMetricsAutoConfiguration::class])
class DemoApplication {
    companion object {
        val logger: Logger = LoggerFactory.getLogger(DemoApplication::class.java)
    }

    @Bean
    fun loggingMeterRegistry() = LoggingMeterRegistry()

// @Bean
// fun kafkaProducerBean(): Producer<String, String> = kafkaProducer()

// @Bean
// fun kafkaAdminClientBean(): Admin = kafkaAdminClient()

    @Bean
    fun beanClass(kafkaTemplate: KafkaTemplate<String, String>): BeanClass {
        logger.info("UsingKafkaTemplate")
        kafkaTemplate.partitionsFor("my-topic")
        kafkaTemplate.send("my-topic", "my-template-message")
        return BeanClass()
    }

    class BeanClass
}

fun main() {
    val rustKafkaProducer: RustKafkaProducer = DefaultRustKafkaProducer(mapOf("bootstrap.servers" to "localhost:9092"))

    val topics = rustKafkaProducer.topics()
    println(topics)

    rustKafkaProducer.send(
        "quickstart-events",
        "key",
        "payload".toByteArray()
    )

    Thread.sleep(1_000)

    rustKafkaProducer.close()

//    kafkaProducer()
//    runApplication<DemoApplication>(*args)
}
