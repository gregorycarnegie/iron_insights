package com.gregorycarnegie.ironinsights.ui.programmes

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.ExerciseDefinitionDao
import com.gregorycarnegie.ironinsights.data.db.dao.PlannedSessionDao
import com.gregorycarnegie.ironinsights.data.db.dao.ProgrammeDao
import com.gregorycarnegie.ironinsights.data.db.dao.TrainingStatsDao
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedExercise
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSession
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet
import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.data.db.relation.PlannedSessionWithExercises
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeLiftBaseline
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionEngine
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionPlan
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class ProgrammeViewModel(
    private val programmeDao: ProgrammeDao,
    private val plannedSessionDao: PlannedSessionDao,
    private val exerciseDefinitionDao: ExerciseDefinitionDao,
    private val trainingStatsDao: TrainingStatsDao,
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    var uiState by mutableStateOf(ProgrammeUiState())
        private set

    private var selectedProgrammeJob: Job? = null

    init {
        loadAll()
    }

    fun loadAll() {
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.getAll().collect { programmes ->
                uiState = uiState.copy(programmes = programmes, isLoading = false)
            }
        }
    }

    fun selectProgramme(id: Long) {
        selectedProgrammeJob?.cancel()
        selectedProgrammeJob = viewModelScope.launch(Dispatchers.IO) {
            programmeDao.getById(id).collect { programmeWithBlocks ->
                val prescriptionSnapshot = buildPrescriptionSnapshot(programmeWithBlocks)
                uiState = uiState.copy(
                    selectedProgramme = programmeWithBlocks,
                    liftBaselines = prescriptionSnapshot.baselines,
                    generatedPlan = prescriptionSnapshot.plan,
                    weightUnit = prescriptionSnapshot.weightUnit,
                )
            }
        }
    }

    fun clearSelection() {
        selectedProgrammeJob?.cancel()
        uiState = uiState.copy(
            selectedProgramme = null,
            liftBaselines = emptyList(),
            generatedPlan = null,
        )
    }

    fun createProgramme(name: String, notes: String? = null) {
        viewModelScope.launch(Dispatchers.IO) {
            val programme = Programme(
                name = name,
                notes = notes,
                createdAtEpochMs = System.currentTimeMillis(),
            )
            val newId = programmeDao.insertProgramme(programme)
            selectProgramme(newId)
        }
    }

    fun deleteProgramme(programme: Programme) {
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.deleteProgramme(programme)
            if (uiState.selectedProgramme?.programme?.id == programme.id) {
                uiState = uiState.copy(
                    selectedProgramme = null,
                    liftBaselines = emptyList(),
                    generatedPlan = null,
                )
            }
        }
    }

    fun addBlock(programmeId: Long, name: String, blockType: String, weekCount: Int) {
        viewModelScope.launch(Dispatchers.IO) {
            val currentBlocks = uiState.selectedProgramme?.blocks?.size ?: 0
            val block = ProgrammeBlock(
                programmeId = programmeId,
                name = name,
                blockType = blockType,
                orderIndex = currentBlocks,
                weekCount = weekCount,
            )
            programmeDao.insertBlock(block)
        }
    }

    fun setActiveProgramme(programmeId: Long) {
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.setActiveProgramme(programmeId)
        }
    }

    fun addBlockSequence(programmeId: Long, blocks: List<ProgrammeBlockDraft>) {
        if (blocks.isEmpty()) return

        viewModelScope.launch(Dispatchers.IO) {
            val baseIndex = uiState.selectedProgramme?.blocks?.size ?: 0
            blocks.forEachIndexed { offset, draft ->
                programmeDao.insertBlock(
                    ProgrammeBlock(
                        programmeId = programmeId,
                        name = draft.name,
                        blockType = draft.blockType,
                        orderIndex = baseIndex + offset,
                        weekCount = draft.weekCount,
                    ),
                )
            }
        }
    }

    fun deleteBlock(block: ProgrammeBlock) {
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.deleteBlock(block)
        }
    }

    fun addSession(blockId: Long, title: String?, dayOfWeek: Int?, weekIndex: Int) {
        viewModelScope.launch(Dispatchers.IO) {
            val session = PlannedSession(
                blockId = blockId,
                title = title,
                dayOfWeek = dayOfWeek,
                weekIndex = weekIndex,
            )
            plannedSessionDao.insertSession(session)
        }
    }

    fun deleteSession(session: PlannedSession) {
        viewModelScope.launch(Dispatchers.IO) {
            plannedSessionDao.deleteSession(session)
        }
    }

    fun addExercise(sessionId: Long, exerciseId: Long, orderIndex: Int) {
        viewModelScope.launch(Dispatchers.IO) {
            val exercise = PlannedExercise(
                plannedSessionId = sessionId,
                exerciseId = exerciseId,
                orderIndex = orderIndex,
            )
            plannedSessionDao.insertExercise(exercise)
        }
    }

    fun deleteExercise(exercise: PlannedExercise) {
        viewModelScope.launch(Dispatchers.IO) {
            plannedSessionDao.deleteExercise(exercise)
        }
    }

    fun addSet(
        plannedExerciseId: Long,
        setIndex: Int,
        prescriptionType: String,
        targetReps: Int,
        targetPercent1rm: Float? = null,
        targetRpe: Float? = null,
        targetSets: Int = 1,
    ) {
        viewModelScope.launch(Dispatchers.IO) {
            val set = PlannedSet(
                plannedExerciseId = plannedExerciseId,
                setIndex = setIndex,
                prescriptionType = prescriptionType,
                targetReps = targetReps,
                targetPercent1rm = targetPercent1rm,
                targetRpe = targetRpe,
                targetSets = targetSets,
            )
            plannedSessionDao.insertSet(set)
        }
    }

    fun deleteSet(set: PlannedSet) {
        viewModelScope.launch(Dispatchers.IO) {
            plannedSessionDao.deleteSet(set)
        }
    }

    fun getSessionsForBlock(blockId: Long): Flow<List<PlannedSessionWithExercises>> {
        return plannedSessionDao.getByBlockId(blockId)
    }

    private suspend fun buildPrescriptionSnapshot(
        programmeWithBlocks: ProgrammeWithBlocks?,
    ): PrescriptionSnapshot {
        val preferences = preferencesRepository.preferencesFlow.firstOrNull()
        val baselines = listOf(
            loadBaseline("S", "Squat"),
            loadBaseline("B", "Bench"),
            loadBaseline("D", "Deadlift"),
        )
        val generatedPlan = buildGeneratedPlan(
            programmeWithBlocks = programmeWithBlocks,
            baselines = baselines,
            roundingIncrementKg = preferences?.roundingIncrement ?: 2.5f,
        )
        return PrescriptionSnapshot(
            baselines = baselines,
            plan = generatedPlan,
            weightUnit = preferences?.weightUnit ?: WeightUnit.KG,
        )
    }

    private suspend fun loadBaseline(
        canonicalLift: String,
        label: String,
    ): ProgrammeLiftBaseline {
        val exercise = exerciseDefinitionDao.getByCanonicalLift(canonicalLift).firstOrNull()?.firstOrNull()
        val e1rmKg = exercise?.let { trainingStatsDao.getLatestE1rm(it.id).firstOrNull() }
        return ProgrammeLiftBaseline(
            canonicalLift = canonicalLift,
            label = label,
            e1rmKg = e1rmKg,
        )
    }

    private fun buildGeneratedPlan(
        programmeWithBlocks: ProgrammeWithBlocks?,
        baselines: List<ProgrammeLiftBaseline>,
        roundingIncrementKg: Float,
    ): ProgrammePrescriptionPlan? {
        val blocks = programmeWithBlocks?.blocks
            ?.sortedBy { it.orderIndex }
            ?.takeIf { it.isNotEmpty() }
            ?: return null

        return ProgrammePrescriptionEngine.build(
            blocks = blocks,
            baselines = baselines,
            roundingIncrementKg = roundingIncrementKg,
        )
    }

    private data class PrescriptionSnapshot(
        val baselines: List<ProgrammeLiftBaseline>,
        val plan: ProgrammePrescriptionPlan?,
        val weightUnit: WeightUnit,
    )

    companion object {
        fun factory(
            programmeDao: ProgrammeDao,
            plannedSessionDao: PlannedSessionDao,
            exerciseDefinitionDao: ExerciseDefinitionDao,
            trainingStatsDao: TrainingStatsDao,
            preferencesRepository: UserPreferencesRepository,
        ): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return ProgrammeViewModel(
                        programmeDao = programmeDao,
                        plannedSessionDao = plannedSessionDao,
                        exerciseDefinitionDao = exerciseDefinitionDao,
                        trainingStatsDao = trainingStatsDao,
                        preferencesRepository = preferencesRepository,
                    ) as T
                }
            }
        }
    }
}
