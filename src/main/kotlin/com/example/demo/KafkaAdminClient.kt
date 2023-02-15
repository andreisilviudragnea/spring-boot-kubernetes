package com.example.demo

import org.apache.kafka.clients.admin.Admin
import org.apache.kafka.clients.admin.AdminClientConfig
import java.util.*

fun kafkaAdminClient(): Admin {
    // TODO: Expose all Kafka metrics
    // No metadata metrics

    val adminClient = Admin.create(
        Properties().also {
            it[AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG] =
                "my-cluster-kafka-0.my-cluster-kafka-brokers.kafka.svc:9092"
            it[AdminClientConfig.METADATA_MAX_AGE_CONFIG] = 15_000
        }
    )

    // 1. DNS resolution in constructor call thread (ClientUtils.parseAndValidateAddresses)
    // 2. DNS resolution in admin thread (Resolved host)
    // 3. API_VERSIONS request in admin thread
    // 4. METADATA request in admin thread (topics=[])

    Thread.sleep(5_000)

    println("list topics calls")
    adminClient.listTopics()

    // 1. DNS resolution in admin thread (Resolved host)
    // 2. API_VERSIONS request in admin thread
    // 3. METADATA request in admin thread (topics=null)

    println("metadata refresh")

    // 1. METADATA request in admin thread (topics=[])

    Thread.sleep(10_000)

    return adminClient
}
