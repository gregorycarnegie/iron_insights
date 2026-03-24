package com.gregorycarnegie.ironinsights.data.db.relation

import androidx.room.Embedded
import androidx.room.Relation
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry

data class ExerciseWithSets(
    @Embedded val exercise: ExercisePerformed,
    @Relation(
        parentColumn = "id",
        entityColumn = "exercisePerformedId",
    )
    val sets: List<SetEntry>,
)
