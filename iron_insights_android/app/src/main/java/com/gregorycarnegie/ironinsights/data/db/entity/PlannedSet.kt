package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "planned_sets",
    foreignKeys = [ForeignKey(entity = PlannedExercise::class, parentColumns = ["id"], childColumns = ["plannedExerciseId"], onDelete = ForeignKey.CASCADE)],
    indices = [Index("plannedExerciseId")],
)
data class PlannedSet(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val plannedExerciseId: Long,
    val setIndex: Int,
    val prescriptionType: String,    // "percent_1rm", "rpe", "fixed_weight"
    val targetPercent1rm: Float? = null,
    val targetRpe: Float? = null,
    val targetReps: Int,
    val targetSets: Int = 1,
)
