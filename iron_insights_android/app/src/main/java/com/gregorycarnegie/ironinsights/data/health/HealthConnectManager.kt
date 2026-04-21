package com.gregorycarnegie.ironinsights.data.health

import android.content.Context
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.permission.HealthPermission
import androidx.health.connect.client.records.ExerciseSessionRecord
import androidx.health.connect.client.records.WeightRecord
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter
import java.time.Instant

class HealthConnectManager(private val context: Context) {

    private val client: HealthConnectClient? by lazy {
        if (isAvailable()) HealthConnectClient.getOrCreate(context) else null
    }

    val requiredPermissions = setOf(
        HealthPermission.getWritePermission(ExerciseSessionRecord::class),
        HealthPermission.getReadPermission(WeightRecord::class),
    )

    fun isAvailable(): Boolean {
        return HealthConnectClient.getSdkStatus(context) == HealthConnectClient.SDK_AVAILABLE
    }

    suspend fun hasAllPermissions(): Boolean {
        val client = client ?: return false
        val granted = client.permissionController.getGrantedPermissions()
        return requiredPermissions.all { it in granted }
    }

    suspend fun writeExerciseSession(record: ExerciseSessionRecord) {
        val client = client ?: return
        client.insertRecords(listOf(record))
    }

    suspend fun readLatestBodyWeight(): Float? {
        val client = client ?: return null
        val now = Instant.now()
        val thirtyDaysAgo = now.minusSeconds(30L * 24 * 60 * 60)
        val response = client.readRecords(
            ReadRecordsRequest(
                recordType = WeightRecord::class,
                timeRangeFilter = TimeRangeFilter.between(thirtyDaysAgo, now),
            ),
        )
        return response.records
            .maxByOrNull { it.time }
            ?.weight
            ?.inKilograms
            ?.toFloat()
    }
}
