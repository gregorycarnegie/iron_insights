package com.gregorycarnegie.ironinsights.data.db.relation

import androidx.room.Embedded
import androidx.room.Relation
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedExercise
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSession
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet

data class PlannedSessionWithExercises(
    @Embedded val session: PlannedSession,
    @Relation(
        parentColumn = "id",
        entityColumn = "plannedSessionId",
        entity = PlannedExercise::class,
    )
    val exercises: List<PlannedExerciseWithSets>,
)

data class PlannedExerciseWithSets(
    @Embedded val exercise: PlannedExercise,
    @Relation(parentColumn = "id", entityColumn = "plannedExerciseId")
    val sets: List<PlannedSet>,
)
