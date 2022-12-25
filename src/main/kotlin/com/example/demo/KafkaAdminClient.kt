package com.example.demo

import org.apache.kafka.clients.admin.Admin
import org.apache.kafka.clients.admin.AdminClientConfig
import java.util.*

fun kafkaAdminClient() {
    val adminClient = Admin.create(Properties().also {
        it[AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG] = "localhost:52781"
    })

    // 1. DNS resolution in constructor call thread (ClientUtils.parseAndValidateAddresses)
    // 2. DNS resolution in admin thread
    // 3. API_VERSIONS request in admin thread
    // 4. METADATA request in admin thread (all topics - empty list)

    Thread.sleep(3600_000)
}
