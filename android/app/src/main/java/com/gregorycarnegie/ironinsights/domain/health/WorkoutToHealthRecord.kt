package com.gregorycarnegie.ironinsights.domain.health

import androidx.health.connect.client.records.ExerciseSessionRecord
import androidx.health.connect.client.records.metadata.Device
import androidx.health.connect.client.records.metadata.Metadata
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession
import java.time.Instant
import java.time.ZoneOffset

object WorkoutToHealthRecord {

    fun convert(session: WorkoutSession): ExerciseSessionRecord? {
        val startMs = session.startedAtEpochMs
        val endMs = session.finishedAtEpochMs ?: return null

        val startInstant = Instant.ofEpochMilli(startMs)
        val endInstant = Instant.ofEpochMilli(endMs)
        val device = Device(type = Device.TYPE_PHONE)
        val metadata = Metadata.activelyRecorded(device)

        return ExerciseSessionRecord(
            startTime = startInstant,
            startZoneOffset = ZoneOffset.systemDefault().rules.getOffset(startInstant),
            endTime = endInstant,
            endZoneOffset = ZoneOffset.systemDefault().rules.getOffset(endInstant),
            exerciseType = ExerciseSessionRecord.EXERCISE_TYPE_WEIGHTLIFTING,
            title = session.title ?: "Iron Insights Workout",
            notes = session.notes,
            metadata = metadata,
        )
    }
}
