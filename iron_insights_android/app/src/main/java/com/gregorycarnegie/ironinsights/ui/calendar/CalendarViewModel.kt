package com.gregorycarnegie.ironinsights.ui.calendar

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.ProgrammeDao
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

data class CalendarUiState(
    val blocks: List<ProgrammeBlock> = emptyList(),
    val isLoading: Boolean = false,
)

class CalendarViewModel(
    private val programmeDao: ProgrammeDao,
) : ViewModel() {

    var uiState by mutableStateOf(CalendarUiState())
        private set

    init {
        loadBlocks()
    }

    private fun loadBlocks() {
        uiState = uiState.copy(isLoading = true)
        viewModelScope.launch(Dispatchers.IO) {
            programmeDao.getActive().collect { activeProgramme ->
                if (activeProgramme != null) {
                    programmeDao.getById(activeProgramme.id).collect { programmeWithBlocks ->
                        val blocks = programmeWithBlocks?.blocks?.sortedBy { it.orderIndex } ?: emptyList()
                        uiState = uiState.copy(blocks = blocks, isLoading = false)
                    }
                } else {
                    uiState = uiState.copy(blocks = emptyList(), isLoading = false)
                }
            }
        }
    }

    companion object {
        fun factory(programmeDao: ProgrammeDao): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return CalendarViewModel(programmeDao) as T
                }
            }
        }
    }
}
