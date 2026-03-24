package com.gregorycarnegie.ironinsights.ui.programmes

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.PlannedSessionDao
import com.gregorycarnegie.ironinsights.data.db.dao.ProgrammeDao
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedExercise
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSession
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet
import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import com.gregorycarnegie.ironinsights.data.db.relation.PlannedSessionWithExercises
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch

class ProgrammeViewModel(
    private val programmeDao: ProgrammeDao,
    private val plannedSessionDao: PlannedSessionDao,
) : ViewModel() {

    var uiState by mutableStateOf(ProgrammeUiState())
        private set

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
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.getById(id).collect { programmeWithBlocks ->
                uiState = uiState.copy(selectedProgramme = programmeWithBlocks)
            }
        }
    }

    fun clearSelection() {
        uiState = uiState.copy(selectedProgramme = null)
    }

    fun createProgramme(name: String, notes: String? = null) {
        viewModelScope.launch(Dispatchers.IO) {
            val programme = Programme(
                name = name,
                notes = notes,
                createdAtEpochMs = System.currentTimeMillis(),
            )
            programmeDao.insertProgramme(programme)
        }
    }

    fun deleteProgramme(programme: Programme) {
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.deleteProgramme(programme)
            if (uiState.selectedProgramme?.programme?.id == programme.id) {
                uiState = uiState.copy(selectedProgramme = null)
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

    companion object {
        fun factory(
            programmeDao: ProgrammeDao,
            plannedSessionDao: PlannedSessionDao,
        ): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return ProgrammeViewModel(programmeDao, plannedSessionDao) as T
                }
            }
        }
    }
}
