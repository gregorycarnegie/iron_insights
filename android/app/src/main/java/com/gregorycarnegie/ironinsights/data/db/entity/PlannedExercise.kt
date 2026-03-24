package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "planned_exercises",
    foreignKeys = [
        ForeignKey(entity = PlannedSession::class, parentColumns = ["id"], childColumns = ["plannedSessionId"], onDelete = ForeignKey.CASCADE),
        ForeignKey(entity = ExerciseDefinition::class, parentColumns = ["id"], childColumns = ["exerciseId"]),
    ],
    indices = [Index("plannedSessionId"), Index("exerciseId")],
)
data class PlannedExercise(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val plannedSessionId: Long,
    val exerciseId: Long,
    val orderIndex: Int,
    val notes: String? = null,
)
