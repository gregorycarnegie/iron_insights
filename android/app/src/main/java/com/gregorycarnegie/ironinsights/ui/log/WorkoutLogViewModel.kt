package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.ExerciseDefinitionDao
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import com.gregorycarnegie.ironinsights.data.repository.TrainingRepository
import com.gregorycarnegie.ironinsights.domain.training.VolumeCalculator
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class WorkoutLogViewModel(
    private val repository: TrainingRepository,
    private val exerciseDao: ExerciseDefinitionDao,
    private val onSessionFinished: ((sessionId: Long) -> Unit)? = null,
) : ViewModel() {

    var uiState by mutableStateOf(WorkoutLogUiState())
        private set

    private var restTimerJob: Job? = null

    init {
        viewModelScope.launch {
            repository.getActiveSession().collect { session ->
                uiState = uiState.copy(activeSession = session)
                if (session != null) {
                    loadSessionExercises(session.id)
                } else {
                    uiState = uiState.copy(exercises = emptyList(), sessionVolume = null)
                }
            }
        }
        viewModelScope.launch {
            exerciseDao.getAll().collect { exercises ->
                uiState = uiState.copy(exerciseLibrary = exercises)
            }
        }
    }

    private fun loadSessionExercises(sessionId: Long) {
        viewModelScope.launch {
            repository.getSessionWithExercises(sessionId).collect { sessionWithExercises ->
                val exercises = sessionWithExercises?.exercises ?: emptyList()
                val volume = computeSessionVolume(exercises)
                uiState = uiState.copy(exercises = exercises, sessionVolume = volume)
            }
        }
    }

    private fun computeSessionVolume(
        exercises: List<com.gregorycarnegie.ironinsights.data.db.relation.ExerciseWithSets>,
    ): VolumeCalculator.SessionVolume {
        val allSets = exercises.flatMap { exerciseWithSets ->
            exerciseWithSets.sets.map { set ->
                VolumeCalculator.SetData(
                    weightKg = set.weightKg,
                    reps = set.reps,
                    isWarmup = set.isWarmup,
                )
            }
        }
        return VolumeCalculator.compute(allSets)
    }

    fun startNewSession() {
        viewModelScope.launch {
            uiState = uiState.copy(isLoading = true)
            repository.startSession(title = null, bodyweightKg = null)
            uiState = uiState.copy(isLoading = false)
        }
    }

    fun finishSession() {
        val session = uiState.activeSession ?: return
        viewModelScope.launch {
            repository.finishSession(session.id)
            stopRestTimer()
            onSessionFinished?.invoke(session.id)
        }
    }

    fun showExercisePicker() {
        uiState = uiState.copy(showExercisePicker = true)
    }

    fun hideExercisePicker() {
        uiState = uiState.copy(showExercisePicker = false, exerciseSearchQuery = "")
    }

    fun setExerciseSearchQuery(query: String) {
        uiState = uiState.copy(exerciseSearchQuery = query)
    }

    fun addExercise(exerciseId: Long) {
        val session = uiState.activeSession ?: return
        viewModelScope.launch {
            repository.addExercise(session.id, exerciseId)
            hideExercisePicker()
        }
    }

    fun logSet(exercisePerformedId: Long, weightKg: Float, reps: Int, rpe: Float?) {
        viewModelScope.launch {
            repository.logSet(
                exercisePerformedId = exercisePerformedId,
                weightKg = weightKg,
                reps = reps,
                rpe = rpe,
                isWarmup = false,
            )
        }
    }

    fun deleteSet(set: SetEntry) {
        viewModelScope.launch {
            repository.deleteSet(set)
        }
    }

    fun removeExercise(exercise: ExercisePerformed) {
        viewModelScope.launch {
            repository.deleteExercise(exercise)
        }
    }

    fun startRestTimer(seconds: Int) {
        stopRestTimer()
        uiState = uiState.copy(restTimerSeconds = seconds, restTimerRunning = true)
        restTimerJob = viewModelScope.launch {
            var remaining = seconds
            while (remaining > 0) {
                delay(1000L)
                remaining--
                uiState = uiState.copy(restTimerSeconds = remaining)
            }
            uiState = uiState.copy(restTimerRunning = false)
        }
    }

    fun stopRestTimer() {
        restTimerJob?.cancel()
        restTimerJob = null
        uiState = uiState.copy(restTimerSeconds = 0, restTimerRunning = false)
    }

    companion object {
        fun factory(
            repository: TrainingRepository,
            exerciseDao: ExerciseDefinitionDao,
            onSessionFinished: ((sessionId: Long) -> Unit)? = null,
        ): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    if (modelClass.isAssignableFrom(WorkoutLogViewModel::class.java)) {
                        return WorkoutLogViewModel(repository, exerciseDao, onSessionFinished) as T
                    }
                    throw IllegalArgumentException("Unknown ViewModel class")
                }
            }
    }
}
