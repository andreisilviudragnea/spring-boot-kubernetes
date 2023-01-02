package com.example.demo

import io.micrometer.core.instrument.logging.LoggingMeterRegistry
import org.apache.kafka.clients.producer.Producer
import org.springframework.boot.actuate.autoconfigure.metrics.KafkaMetricsAutoConfiguration
import org.springframework.boot.autoconfigure.AutoConfiguration
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication
import org.springframework.context.annotation.Bean

@SpringBootApplication
@AutoConfiguration(after = [KafkaMetricsAutoConfiguration::class])
class DemoApplication {
	@Bean
	fun loggingMeterRegistry() = LoggingMeterRegistry()

	@Bean
	fun kafkaProducerBean(): Producer<String, String> = kafkaProducer()

//	@Bean
//	fun kafkaAdminClientBean(): Admin = kafkaAdminClient()
}

fun main(args: Array<String>) {
	runApplication<DemoApplication>(*args)
}
