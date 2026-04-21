package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "workout_sessions")
data class WorkoutSession(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val startedAtEpochMs: Long,
    val finishedAtEpochMs: Long? = null,
    val title: String? = null,
    val notes: String? = null,
    val programmeSessionId: Long? = null,
    val bodyweightKg: Float? = null,
)
