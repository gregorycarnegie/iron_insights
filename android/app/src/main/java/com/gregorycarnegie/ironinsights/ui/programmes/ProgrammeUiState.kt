package com.gregorycarnegie.ironinsights.ui.programmes

import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks

data class ProgrammeUiState(
    val programmes: List<Programme> = emptyList(),
    val selectedProgramme: ProgrammeWithBlocks? = null,
    val isLoading: Boolean = false,
)
