package com.example.demo

import org.apache.kafka.clients.admin.Admin
import org.apache.kafka.clients.admin.AdminClientConfig
import java.util.*

fun kafkaAdminClient() {
    // TODO: Inspect metadata metrics

    val adminClient = Admin.create(Properties().also {
        it[AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG] = "my-cluster-kafka-0.my-cluster-kafka-brokers.kafka.svc:9092"
        // TODO: Metadata refresh
    })

    // 1. DNS resolution in constructor call thread (ClientUtils.parseAndValidateAddresses)
    // 2. DNS resolution in admin thread
    // 3. API_VERSIONS request in admin thread
    // 4. METADATA request in admin thread (topics=[])

    Thread.sleep(5_000)

    println("list topics calls")
    adminClient.listTopics()

    // 1. DNS resolution in admin thread
    // 2. API_VERSIONS request in admin thread
    // 3. METADATA request in admin thread (topics=null)

    Thread.sleep(3600_000)
}
