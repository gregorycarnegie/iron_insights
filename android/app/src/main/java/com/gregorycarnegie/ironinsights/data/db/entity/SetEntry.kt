package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "set_entries",
    foreignKeys = [
        ForeignKey(
            entity = ExercisePerformed::class,
            parentColumns = ["id"],
            childColumns = ["exercisePerformedId"],
            onDelete = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("exercisePerformedId")],
)
data class SetEntry(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val exercisePerformedId: Long,
    val setIndex: Int,
    val weightKg: Float,
    val reps: Int,
    val rpe: Float? = null,
    val rir: Int? = null,
    val isWarmup: Boolean = false,
    val isPersonalRecord: Boolean = false,
    val e1rmKg: Float? = null,
    val completedAtEpochMs: Long? = null,
)
