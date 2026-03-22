package com.gregorycarnegie.ironinsights.ui.home

import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSummary
import com.gregorycarnegie.ironinsights.data.model.HistogramLookupPreview
import com.gregorycarnegie.ironinsights.data.model.LatestJson
import com.gregorycarnegie.ironinsights.data.model.SliceIndexPreview
import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.model.TrendsPreview

data class HomeUiState(
    val isLoading: Boolean = false,
    val cachedLatestVersion: String? = null,
    val latest: LatestJson? = null,
    val rootShardCount: Int? = null,
    val shardPreview: SliceIndexPreview? = null,
    val selectorState: LookupSelectorState? = null,
    val lookupPreview: HistogramLookupPreview? = null,
    val trendsPreview: TrendsPreview? = null,
    val trendSeries: TrendSeriesPresentation? = null,
    val comparisonRows: List<CohortComparisonRowPresentation> = emptyList(),
    val loadSummary: DatasetLoadSummary? = null,
    val errorMessage: String? = null,
)

data class TrendSeriesPresentation(
    val key: String,
    val bucket: String,
    val note: String,
    val points: List<TrendPoint>,
    val growthSummary: String,
    val p50DriftSummary: String,
    val p90DriftSummary: String,
)

data class CohortComparisonRowPresentation(
    val id: String,
    val label: String,
    val wc: String,
    val age: String,
    val tested: String,
    val metric: String,
    val total: Int?,
    val totalDelta: Long?,
    val minKg: Float?,
    val maxKg: Float?,
    val status: String,
    val statusOk: Boolean,
    val isCurrent: Boolean,
)

data class LookupFilters(
    val sex: String,
    val equip: String,
    val wc: String,
    val age: String,
    val tested: String,
    val lift: String,
    val metric: String,
)

data class LookupFilterOptions(
    val sexes: List<String>,
    val equips: List<String>,
    val weightClasses: List<String>,
    val ages: List<String>,
    val tested: List<String>,
    val lifts: List<String>,
    val metrics: List<String>,
)

data class LookupSelectorState(
    val filters: LookupFilters,
    val options: LookupFilterOptions,
)

enum class LookupFilterField {
    SEX,
    EQUIP,
    WEIGHT_CLASS,
    AGE,
    TESTED,
    LIFT,
    METRIC,
}
