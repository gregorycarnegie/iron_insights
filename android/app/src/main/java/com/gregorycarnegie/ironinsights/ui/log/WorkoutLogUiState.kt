package com.gregorycarnegie.ironinsights.ui.log

import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession
import com.gregorycarnegie.ironinsights.data.db.relation.ExerciseWithSets
import com.gregorycarnegie.ironinsights.domain.training.VolumeCalculator

data class WorkoutLogUiState(
    val activeSession: WorkoutSession? = null,
    val exercises: List<ExerciseWithSets> = emptyList(),
    val sessionVolume: VolumeCalculator.SessionVolume? = null,
    val isLoading: Boolean = false,
    val showExercisePicker: Boolean = false,
    val exerciseLibrary: List<ExerciseDefinition> = emptyList(),
    val exerciseSearchQuery: String = "",
    val restTimerSeconds: Int = 0,
    val restTimerRunning: Boolean = false,
)
