package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.ExerciseDefinitionDao
import com.gregorycarnegie.ironinsights.data.db.dao.ProgrammeDao
import com.gregorycarnegie.ironinsights.data.db.dao.TrainingStatsDao
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.data.repository.TrainingRepository
import com.gregorycarnegie.ironinsights.domain.training.ProgrammedWorkoutCodec
import com.gregorycarnegie.ironinsights.domain.training.ProgrammedWorkoutRecommendation
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeLiftBaseline
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionEngine
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionPlan
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeSessionPrescription
import com.gregorycarnegie.ironinsights.domain.training.VolumeCalculator
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch

class WorkoutLogViewModel(
    private val repository: TrainingRepository,
    private val exerciseDao: ExerciseDefinitionDao,
    private val programmeDao: ProgrammeDao,
    private val trainingStatsDao: TrainingStatsDao,
    private val preferencesRepository: UserPreferencesRepository,
    private val onSessionFinished: ((sessionId: Long) -> Unit)? = null,
) : ViewModel() {

    var uiState by mutableStateOf(WorkoutLogUiState())
        private set

    private var restTimerJob: Job? = null
    private var sessionExercisesJob: Job? = null
    private var activeProgrammeJob: Job? = null
    private var activeProgrammePlan: ActiveProgrammePlan? = null

    init {
        viewModelScope.launch {
            repository.getActiveSession().collect { session ->
                uiState = uiState.copy(
                    activeSession = session,
                    programmedSessionMetadata = ProgrammedWorkoutCodec.decodeSessionMetadata(session?.notes),
                )
                if (session != null) {
                    loadSessionExercises(session.id)
                } else {
                    sessionExercisesJob?.cancel()
                    uiState = uiState.copy(
                        exercises = emptyList(),
                        sessionVolume = null,
                        programmedExercisePrescriptions = emptyMap(),
                    )
                    recomputeRecommendedWorkout()
                }
            }
        }
        viewModelScope.launch {
            exerciseDao.getAll().collect { exercises ->
                uiState = uiState.copy(exerciseLibrary = exercises)
            }
        }
        viewModelScope.launch {
            repository.getRecentSessionDetails().collect { sessions ->
                uiState = uiState.copy(recentSessions = sessions)
                recomputeRecommendedWorkout()
            }
        }
        viewModelScope.launch {
            programmeDao.getActive().collect { activeProgramme ->
                activeProgrammeJob?.cancel()
                if (activeProgramme == null) {
                    activeProgrammePlan = null
                    uiState = uiState.copy(recommendedProgrammedWorkout = null)
                } else {
                    activeProgrammeJob = launch {
                        programmeDao.getById(activeProgramme.id).collect { programmeWithBlocks ->
                            activeProgrammePlan = buildActiveProgrammePlan(programmeWithBlocks)
                            recomputeRecommendedWorkout()
                        }
                    }
                }
            }
        }
    }

    private fun loadSessionExercises(sessionId: Long) {
        sessionExercisesJob?.cancel()
        sessionExercisesJob = viewModelScope.launch {
            repository.getSessionWithExercises(sessionId).collect { sessionWithExercises ->
                val exercises = sessionWithExercises?.exercises ?: emptyList()
                val volume = computeSessionVolume(exercises)
                val programmedPrescriptions = exercises.mapNotNull { exercise ->
                    ProgrammedWorkoutCodec.decodeExercisePrescription(exercise.exercise.notes)?.let { decoded ->
                        exercise.exercise.id to decoded
                    }
                }.toMap()
                uiState = uiState.copy(
                    exercises = exercises,
                    sessionVolume = volume,
                    programmedExercisePrescriptions = programmedPrescriptions,
                )
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

    fun startProgrammedWorkout() {
        val recommendation = uiState.recommendedProgrammedWorkout ?: return
        if (uiState.exerciseLibrary.isEmpty()) return
        viewModelScope.launch {
            uiState = uiState.copy(isLoading = true)
            val exerciseIdByName = uiState.exerciseLibrary.associateBy({ it.name }, { it.id })
            val sessionId = repository.startSession(
                title = "Week ${recommendation.metadata.weekNumber} • ${recommendation.metadata.dayLabel} • ${recommendation.session.title}",
                notes = ProgrammedWorkoutCodec.encodeSessionMetadata(recommendation.metadata),
                bodyweightKg = null,
            )
            recommendation.session.exercises.forEach { exercise ->
                val exerciseId = exerciseIdByName[exercise.exerciseName] ?: return@forEach
                repository.addExercise(
                    sessionId = sessionId,
                    exerciseId = exerciseId,
                    notes = ProgrammedWorkoutCodec.encodeExercisePrescription(exercise),
                )
            }
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

    private suspend fun buildActiveProgrammePlan(
        programmeWithBlocks: ProgrammeWithBlocks?,
    ): ActiveProgrammePlan? {
        val programme = programmeWithBlocks?.programme ?: return null
        val blocks = programmeWithBlocks.blocks.sortedBy { it.orderIndex }
        if (blocks.isEmpty()) return null

        val preferences = preferencesRepository.preferencesFlow.firstOrNull()
        val baselines = listOf(
            loadBaseline("S", "Squat"),
            loadBaseline("B", "Bench"),
            loadBaseline("D", "Deadlift"),
        )
        val plan = ProgrammePrescriptionEngine.build(
            blocks = blocks,
            baselines = baselines,
            roundingIncrementKg = preferences?.roundingIncrement ?: 2.5f,
        )
        return ActiveProgrammePlan(
            programmeId = programme.id,
            programmeName = programme.name,
            plan = plan,
        )
    }

    private suspend fun loadBaseline(
        canonicalLift: String,
        label: String,
    ): ProgrammeLiftBaseline {
        val exercise = exerciseDao.getByCanonicalLift(canonicalLift).firstOrNull()?.firstOrNull()
        val e1rmKg = exercise?.let { trainingStatsDao.getLatestE1rm(it.id).firstOrNull() }
        return ProgrammeLiftBaseline(
            canonicalLift = canonicalLift,
            label = label,
            e1rmKg = e1rmKg,
        )
    }

    private fun recomputeRecommendedWorkout() {
        val activePlan = activeProgrammePlan
        if (activePlan == null || uiState.activeSession != null) {
            uiState = uiState.copy(recommendedProgrammedWorkout = null)
            return
        }

        val flattenedSessions = flattenProgrammedSessions(activePlan)
        val lastCompleted = uiState.recentSessions
            .asSequence()
            .filter { it.session.finishedAtEpochMs != null }
            .mapNotNull { ProgrammedWorkoutCodec.decodeSessionMetadata(it.session.notes) }
            .firstOrNull { it.programmeId == activePlan.programmeId }

        val nextIndex = if (lastCompleted == null) 0 else lastCompleted.sessionIndex + 1
        uiState = uiState.copy(
            recommendedProgrammedWorkout = flattenedSessions.getOrNull(nextIndex),
        )
    }

    private fun flattenProgrammedSessions(
        activePlan: ActiveProgrammePlan,
    ): List<ProgrammedWorkoutRecommendation> {
        val flattened = mutableListOf<ProgrammedWorkoutRecommendation>()
        activePlan.plan.weeks.forEach { week ->
            week.sessions.forEach { session ->
                flattened += ProgrammedWorkoutRecommendation(
                    metadata = com.gregorycarnegie.ironinsights.domain.training.ProgrammedWorkoutMetadata(
                        programmeId = activePlan.programmeId,
                        programmeName = activePlan.programmeName,
                        weekNumber = week.weekNumber,
                        sessionIndex = flattened.size,
                        dayLabel = session.dayLabel,
                        sessionTitle = session.title,
                    ),
                    session = session,
                )
            }
        }
        return flattened
    }

    private data class ActiveProgrammePlan(
        val programmeId: Long,
        val programmeName: String,
        val plan: ProgrammePrescriptionPlan,
    )

    companion object {
        fun factory(
            repository: TrainingRepository,
            exerciseDao: ExerciseDefinitionDao,
            programmeDao: ProgrammeDao,
            trainingStatsDao: TrainingStatsDao,
            preferencesRepository: UserPreferencesRepository,
            onSessionFinished: ((sessionId: Long) -> Unit)? = null,
        ): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    if (modelClass.isAssignableFrom(WorkoutLogViewModel::class.java)) {
                        return WorkoutLogViewModel(
                            repository = repository,
                            exerciseDao = exerciseDao,
                            programmeDao = programmeDao,
                            trainingStatsDao = trainingStatsDao,
                            preferencesRepository = preferencesRepository,
                            onSessionFinished = onSessionFinished,
                        ) as T
                    }
                    throw IllegalArgumentException("Unknown ViewModel class")
                }
            }
    }
}
