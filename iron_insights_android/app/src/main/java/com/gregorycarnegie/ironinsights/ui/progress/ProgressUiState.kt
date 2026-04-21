package com.gregorycarnegie.ironinsights.ui.progress

import com.gregorycarnegie.ironinsights.domain.progress.E1rmTrendBuilder.E1rmTrend

data class ProgressUiState(
    val trends: List<E1rmTrend> = emptyList(),
    val personalRecords: Map<String, PrSummary> = emptyMap(), // keyed by canonical lift
    val isLoading: Boolean = false,
)

data class PrSummary(
    val liftName: String,
    val bestE1rmKg: Float?,
    val bestWeightKg: Float?,
    val bestReps: Int?,
    val percentileLabel: String? = null,  // from PercentileRankIntegration
)
