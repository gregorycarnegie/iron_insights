package com.gregorycarnegie.ironinsights.ui.programmes

import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeLiftBaseline
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionPlan

data class ProgrammeUiState(
    val programmes: List<Programme> = emptyList(),
    val selectedProgramme: ProgrammeWithBlocks? = null,
    val isLoading: Boolean = false,
    val weightUnit: WeightUnit = WeightUnit.KG,
    val liftBaselines: List<ProgrammeLiftBaseline> = emptyList(),
    val generatedPlan: ProgrammePrescriptionPlan? = null,
)
