package com.gregorycarnegie.ironinsights.data.db.relation

import androidx.room.Embedded
import androidx.room.Relation
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession

data class SessionWithExercises(
    @Embedded val session: WorkoutSession,
    @Relation(
        parentColumn = "id",
        entityColumn = "sessionId",
        entity = ExercisePerformed::class,
    )
    val exercises: List<ExerciseWithSets>,
)
