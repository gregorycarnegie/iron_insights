package com.gregorycarnegie.ironinsights.ui.home

import android.os.Handler
import android.os.Looper
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.gregorycarnegie.ironinsights.data.model.PublishedSliceContract
import com.gregorycarnegie.ironinsights.data.model.PublishedSliceKey
import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSource
import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSummary
import com.gregorycarnegie.ironinsights.data.model.HistogramBin
import com.gregorycarnegie.ironinsights.data.model.HistogramLookupPreview
import com.gregorycarnegie.ironinsights.data.model.HeatmapBin
import com.gregorycarnegie.ironinsights.data.model.LatestJson
import com.gregorycarnegie.ironinsights.data.model.RootIndex
import com.gregorycarnegie.ironinsights.data.model.SliceIndex
import com.gregorycarnegie.ironinsights.data.model.SliceIndexEntries
import com.gregorycarnegie.ironinsights.data.model.SliceIndexPreview
import com.gregorycarnegie.ironinsights.data.model.SliceSummary
import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.model.TrendSeries
import com.gregorycarnegie.ironinsights.data.model.TrendsJson
import com.gregorycarnegie.ironinsights.data.model.TrendsPreview
import com.gregorycarnegie.ironinsights.data.repository.PublishedDataRepository
import com.gregorycarnegie.ironinsights.data.repository.valueForPercentile
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import java.util.Locale
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

data class ProfileLookupDefaults(
    val sex: String = "",
    val equipment: String = "",
    val tested: String = "",
    val bodyweightKg: Float? = null,
    val squatKg: Float? = null,
    val benchKg: Float? = null,
    val deadliftKg: Float? = null,
)

class HomeViewModel(
    private val repository: PublishedDataRepository,
    initialProfileDefaults: ProfileLookupDefaults = ProfileLookupDefaults(),
) : ViewModel() {
    private val executor: ExecutorService = Executors.newSingleThreadExecutor()
    private val mainHandler = Handler(Looper.getMainLooper())
    private var datasetSnapshot: DatasetSnapshot? = null
    private var currentRoute: AppRoute = AppRoute.LOOKUP
    private var profileDefaults: ProfileLookupDefaults = initialProfileDefaults
    private val lookupLiftOverrides = mutableMapOf<String, String>()
    private var lookupBodyweightOverride: String? = null

    var uiState by mutableStateOf(
        HomeUiState(cachedLatestVersion = repository.readCachedLatestVersion()),
    )
        private set

    init {
        refresh()
    }

    fun setRoute(route: AppRoute) {
        val routeChanged = currentRoute != route
        currentRoute = route
        if (!routeChanged || uiState.isLoading || !stateNeedsReloadForRoute(uiState, route)) {
            return
        }

        reloadCurrentSelection(loadModeForRoute(route))
    }

    fun refresh() {
        if (uiState.isLoading) {
            return
        }

        val previousState = uiState
        uiState = uiState.copy(isLoading = true, errorMessage = null)
        val loadMode = loadModeForRoute(currentRoute)

        executor.execute {
            val nextState = loadBootstrap(previousState, loadMode)
            mainHandler.post {
                applyLoadedState(nextState, loadMode)
            }
        }
    }

    fun updateFilter(
        field: LookupFilterField,
        value: String,
    ) {
        if (uiState.isLoading) {
            return
        }

        val snapshot = datasetSnapshot ?: return
        val currentFilters = uiState.selectorState?.filters ?: defaultFilters(snapshot.rootIndex, profileDefaults)
        val requestedFilters = currentFilters.updated(field, value)
        val loadMode = loadModeForRoute(currentRoute)

        uiState = uiState.copy(isLoading = true, errorMessage = null)
        executor.execute {
            val nextState = buildHomeState(
                snapshot = snapshot,
                requestedFilters = requestedFilters,
                repository = repository,
                loadMode = loadMode,
            )
            mainHandler.post {
                applyLoadedState(nextState, loadMode)
            }
        }
    }

    fun updateLookupLiftInput(value: String) {
        val filters = uiState.selectorState?.filters ?: return
        lookupLiftOverrides[lookupInputKey(filters)] = value
        uiState = resolveLookupInputState(uiState)
    }

    fun updateLookupBodyweightInput(value: String) {
        lookupBodyweightOverride = value
        uiState = resolveLookupInputState(uiState)
    }

    fun resetLookupInputsToProfile() {
        val filters = uiState.selectorState?.filters
        if (filters != null && profileLiftInputFor(filters).isNotBlank()) {
            lookupLiftOverrides.remove(lookupInputKey(filters))
        }
        if (profileBodyweightInput().isNotBlank()) {
            lookupBodyweightOverride = null
        }
        uiState = resolveLookupInputState(uiState)
    }

    fun syncProfileDefaults(updatedProfileDefaults: ProfileLookupDefaults) {
        profileDefaults = updatedProfileDefaults
        uiState = resolveLookupInputState(uiState)
    }

    private fun loadBootstrap(
        previousState: HomeUiState,
        loadMode: LookupPayloadLoadMode,
    ): HomeUiState {
        val latestLoaded = repository.fetchLatest().getOrElse { error ->
            datasetSnapshot = null
            return previousState.copy(
                isLoading = false,
                cachedLatestVersion = repository.readCachedLatestVersion(),
                errorMessage = error.message ?: "Failed to load latest dataset pointer.",
            )
        }
        val latest = latestLoaded.value

        val rootIndexLoaded = repository.fetchRootIndex(latest.version).getOrElse { error ->
            datasetSnapshot = null
            return HomeUiState(
                isLoading = false,
                cachedLatestVersion = latest.version,
                latest = latest,
                loadSummary = DatasetLoadSummary(latest = latestLoaded.source),
                rootShardCount = null,
                shardPreview = null,
                errorMessage = error.message ?: "Failed to load root index.",
            )
        }
        val rootIndex = rootIndexLoaded.value

        val trendsLoaded = repository.fetchTrends(latest.version).getOrNull()
        val trendsPayload = trendsLoaded?.value
        val trendsPreview = trendsPayload?.let(::buildTrendsPreview)

        val snapshot = DatasetSnapshot(
            latest = latest,
            latestSource = latestLoaded.source,
            rootIndex = rootIndex,
            rootIndexSource = rootIndexLoaded.source,
            trendsPreview = trendsPreview,
            trendsSource = trendsLoaded?.source,
            trendBucket = trendsPayload?.bucket,
            trendSeries = trendsPayload?.series ?: emptyList(),
        )
        datasetSnapshot = snapshot
        return buildHomeState(
            snapshot = snapshot,
            requestedFilters = defaultFilters(rootIndex, profileDefaults),
            repository = repository,
            loadMode = loadMode,
        )
    }

    private fun reloadCurrentSelection(loadMode: LookupPayloadLoadMode = loadModeForRoute(currentRoute)) {
        val snapshot = datasetSnapshot ?: return
        val requestedFilters = uiState.selectorState?.filters ?: defaultFilters(snapshot.rootIndex, profileDefaults)

        uiState = uiState.copy(isLoading = true, errorMessage = null)
        executor.execute {
            val nextState = buildHomeState(
                snapshot = snapshot,
                requestedFilters = requestedFilters,
                repository = repository,
                loadMode = loadMode,
            )
            mainHandler.post {
                applyLoadedState(nextState, loadMode)
            }
        }
    }

    private fun applyLoadedState(
        nextState: HomeUiState,
        completedLoadMode: LookupPayloadLoadMode,
    ) {
        uiState = resolveLookupInputState(nextState)
        val desiredLoadMode = loadModeForRoute(currentRoute)
        if (completedLoadMode == desiredLoadMode || !stateNeedsReloadForRoute(nextState, currentRoute)) {
            return
        }

        reloadCurrentSelection(desiredLoadMode)
    }

    private fun stateNeedsReloadForRoute(
        state: HomeUiState,
        route: AppRoute,
    ): Boolean {
        if (datasetSnapshot == null || state.selectorState == null) {
            return false
        }

        return when (loadModeForRoute(route)) {
            LookupPayloadLoadMode.FULL -> state.lookupPreview == null
            LookupPayloadLoadMode.SUMMARY_ONLY -> {
                state.lookupPreview != null ||
                    state.loadSummary?.histogram != null ||
                    state.loadSummary?.heatmap != null
            }
        }
    }

    override fun onCleared() {
        executor.shutdownNow()
    }

    companion object {
        fun factory(
            repository: PublishedDataRepository,
            profileDefaults: ProfileLookupDefaults = ProfileLookupDefaults(),
        ): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    if (modelClass.isAssignableFrom(HomeViewModel::class.java)) {
                        return HomeViewModel(repository, profileDefaults) as T
                    }
                    throw IllegalArgumentException("Unknown ViewModel class: ${modelClass.name}")
                }
            }
    }

    private fun resolveLookupInputState(state: HomeUiState): HomeUiState {
        val filters = state.selectorState?.filters
        val profileLiftInput = filters?.let(::profileLiftInputFor).orEmpty()
        val liftInput = when {
            filters == null -> ""
            lookupLiftOverrides.containsKey(lookupInputKey(filters)) -> {
                lookupLiftOverrides.getValue(lookupInputKey(filters))
            }
            profileLiftInput.isNotBlank() -> profileLiftInput
            else -> state.lookupPreview?.p50?.let(::formatMetricValue).orEmpty()
        }

        val profileBodyweightInput = profileBodyweightInput()
        val bodyweightInput = lookupBodyweightOverride ?: profileBodyweightInput

        return state.copy(
            lookupInputState = LookupInputState(
                liftInput = liftInput,
                bodyweightInput = bodyweightInput,
                profileLiftInput = profileLiftInput,
                profileBodyweightInput = profileBodyweightInput,
                hasProfileValues = profileLiftInput.isNotBlank() || profileBodyweightInput.isNotBlank(),
            ),
        )
    }

    private fun lookupInputKey(filters: LookupFilters): String {
        return "${filters.lift}|${filters.metric}"
    }

    private fun profileBodyweightInput(): String {
        return profileDefaults.bodyweightKg?.let(::formatMetricValue).orEmpty()
    }

    private fun profileLiftInputFor(filters: LookupFilters): String {
        val liftKg = profileLiftValueKg(filters.lift) ?: return ""
        return formatProfileValueForMetric(liftKg, filters.metric).orEmpty()
    }

    private fun profileLiftValueKg(lift: String): Float? {
        return when (lift) {
            "S" -> profileDefaults.squatKg
            "B" -> profileDefaults.benchKg
            "D" -> profileDefaults.deadliftKg
            "T" -> {
                val squat = profileDefaults.squatKg
                val bench = profileDefaults.benchKg
                val deadlift = profileDefaults.deadliftKg
                if (squat != null && bench != null && deadlift != null) {
                    squat + bench + deadlift
                } else {
                    null
                }
            }
            else -> null
        }
    }

    private fun formatProfileValueForMetric(valueKg: Float, metric: String): String? {
        return when (metric.lowercase(Locale.US)) {
            "kg" -> formatMetricValue(valueKg)
            "lb" -> formatMetricValue(valueKg * 2.20462f)
            else -> null
        }
    }
}

private data class DatasetSnapshot(
    val latest: LatestJson,
    val latestSource: DatasetLoadSource,
    val rootIndex: RootIndex,
    val rootIndexSource: DatasetLoadSource,
    val trendsPreview: TrendsPreview?,
    val trendsSource: DatasetLoadSource?,
    val trendBucket: String? = null,
    val trendSeries: List<TrendSeries> = emptyList(),
    val sliceIndexByShard: MutableMap<String, SliceIndex> = mutableMapOf(),
    val sliceIndexSourceByShard: MutableMap<String, DatasetLoadSource> = mutableMapOf(),
    val histogramByPath: MutableMap<String, HistogramBin> = mutableMapOf(),
    val histogramSourceByPath: MutableMap<String, DatasetLoadSource> = mutableMapOf(),
    val heatmapByPath: MutableMap<String, HeatmapBin> = mutableMapOf(),
    val heatmapSourceByPath: MutableMap<String, DatasetLoadSource> = mutableMapOf(),
)

private enum class LookupPayloadLoadMode {
    FULL,
    SUMMARY_ONLY,
}

private fun loadModeForRoute(route: AppRoute): LookupPayloadLoadMode {
    return when (route) {
        AppRoute.COMPARE,
        AppRoute.TRENDS,
        AppRoute.CALCULATORS,
        AppRoute.LOG,
        AppRoute.PROGRAMMES,
        AppRoute.PROGRESS,
        AppRoute.PROFILE,
        -> LookupPayloadLoadMode.SUMMARY_ONLY

        AppRoute.LOOKUP -> LookupPayloadLoadMode.FULL
    }
}

internal data class LookupSliceRow(
    val rawKey: String,
    val filters: LookupFilters,
    val histPath: String,
    val heatPath: String,
    val summary: SliceSummary?,
)

private fun buildSliceIndexPreview(
    shardKey: String,
    shardPath: String,
    sliceIndex: SliceIndex,
): SliceIndexPreview {
    return when (val slices = sliceIndex.slices) {
        is SliceIndexEntries.MapEntries -> {
            val sampleEntry = slices.entries.entries.firstOrNull()
            SliceIndexPreview(
                shardKey = shardKey,
                shardPath = shardPath,
                sliceCount = slices.entries.size,
                sampleSliceKey = sampleEntry?.key,
                sampleHistPath = sampleEntry?.value?.hist,
            )
        }

        is SliceIndexEntries.KeyEntries -> {
            SliceIndexPreview(
                shardKey = shardKey,
                shardPath = shardPath,
                sliceCount = slices.keys.size,
                sampleSliceKey = slices.keys.firstOrNull(),
                sampleHistPath = slices.keys.firstOrNull()?.let(::histPathFromSliceKey),
            )
        }
    }
}

private fun buildTrendsPreview(trends: TrendsJson): TrendsPreview {
    val sampleSeries = trends.series.firstOrNull()
    return TrendsPreview(
        bucket = trends.bucket,
        seriesCount = trends.series.size,
        sampleSeriesKey = sampleSeries?.key,
        samplePointCount = sampleSeries?.points?.size,
    )
}

private fun buildLoadSummary(
    snapshot: DatasetSnapshot,
    shardKey: String? = null,
    histogramPath: String? = null,
    heatmapPath: String? = null,
): DatasetLoadSummary {
    return DatasetLoadSummary(
        latest = snapshot.latestSource,
        rootIndex = snapshot.rootIndexSource,
        shardIndex = shardKey?.let { snapshot.sliceIndexSourceByShard[it] },
        histogram = histogramPath?.let { snapshot.histogramSourceByPath[it] },
        heatmap = heatmapPath?.let { snapshot.heatmapSourceByPath[it] },
        trends = snapshot.trendsSource,
    )
}

internal fun buildTrendSeriesPresentation(
    bucket: String,
    series: List<TrendSeries>,
    filters: LookupFilters,
): TrendSeriesPresentation? {
    val key = buildTrendKey(filters)
    val points = series.firstOrNull { it.key == key }?.points ?: return null
    if (points.size < 2) {
        return null
    }

    return TrendSeriesPresentation(
        key = key,
        bucket = bucket,
        note = buildTrendNote(filters, points),
        points = points,
        growthSummary = growthSummary(points),
        p50DriftSummary = driftSummary(points) { it.p50 },
        p90DriftSummary = driftSummary(points) { it.p90 },
    )
}

internal fun buildTrendKey(filters: LookupFilters): String {
    return "sex=${filters.sex}|equip=${filters.equip}|tested=${filters.tested}|lift=${filters.lift}|metric=${filters.metric}"
}

internal fun growthSummary(points: List<TrendPoint>): String {
    val first = points.firstOrNull() ?: return "Not enough points for growth summary."
    val last = points.lastOrNull() ?: return "Not enough points for growth summary."
    val delta = last.total - first.total
    val pct = if (first.total == 0) 0f else (delta.toFloat() / first.total.toFloat()) * 100f
    return String.format(
        Locale.US,
        "%d-%d: %+d lifters (%+.1f%%).",
        first.year,
        last.year,
        delta,
        pct,
    )
}

internal fun driftSummary(
    points: List<TrendPoint>,
    select: (TrendPoint) -> Float,
): String {
    val first = points.firstOrNull() ?: return "Not enough points for drift summary."
    val last = points.lastOrNull() ?: return "Not enough points for drift summary."
    return String.format(
        Locale.US,
        "%d-%d: %+.1f",
        first.year,
        last.year,
        select(last) - select(first),
    )
}

private fun buildTrendNote(
    filters: LookupFilters,
    points: List<TrendPoint>,
): String {
    val first = points.firstOrNull()
    val last = points.lastOrNull()
    if (first == null || last == null) {
        return "Time buckets use yearly snapshots from best-lift records; sparse cohorts may have missing years."
    }

    return String.format(
        Locale.US,
        "Year buckets %d-%d for %s / %s / %s / %s / %s. Cohort size %d -> %d. Trends aggregate across bodyweight and age classes for this slice.",
        first.year,
        last.year,
        trendSexLabel(filters.sex),
        filters.equip,
        trendTestedLabel(filters.tested),
        liftLabel(filters.lift),
        filters.metric,
        first.total,
        last.total,
    )
}

private fun trendSexLabel(value: String): String {
    return when (value) {
        "M" -> "Men"
        "F" -> "Women"
        else -> value
    }
}

private fun trendTestedLabel(value: String): String {
    return when (value) {
        "Yes" -> "Tested"
        "All" -> "All tested statuses"
        else -> value
    }
}

private data class CohortComparisonVariant(
    val label: String,
    val wc: String,
    val age: String,
    val tested: String,
    val metric: String,
    val isCurrent: Boolean,
)

private data class CrossSexRowMatch(
    val row: LookupSliceRow,
    val note: String?,
)

internal fun buildComparisonRows(
    rows: List<LookupSliceRow>,
    filters: LookupFilters,
    baseTotal: Int?,
): List<CohortComparisonRowPresentation> {
    val variants = listOf(
        CohortComparisonVariant(
            label = "Current slice",
            wc = filters.wc,
            age = filters.age,
            tested = filters.tested,
            metric = filters.metric,
            isCurrent = true,
        ),
        CohortComparisonVariant(
            label = "All Ages",
            wc = filters.wc,
            age = "All Ages",
            tested = filters.tested,
            metric = filters.metric,
            isCurrent = false,
        ),
        CohortComparisonVariant(
            label = "All weight classes",
            wc = "All",
            age = filters.age,
            tested = filters.tested,
            metric = filters.metric,
            isCurrent = false,
        ),
        CohortComparisonVariant(
            label = "All tested statuses",
            wc = filters.wc,
            age = filters.age,
            tested = "All",
            metric = filters.metric,
            isCurrent = false,
        ),
    )

    return variants
        .map { variant -> buildComparisonRow(rows, filters, baseTotal, variant) }
        .distinctBy { it.id }
}

private fun buildComparisonRow(
    rows: List<LookupSliceRow>,
    filters: LookupFilters,
    baseTotal: Int?,
    variant: CohortComparisonVariant,
): CohortComparisonRowPresentation {
    val row = rows.firstOrNull {
        it.filters.sex == filters.sex &&
            it.filters.equip == filters.equip &&
            it.filters.wc == variant.wc &&
            it.filters.age == variant.age &&
            it.filters.tested == variant.tested &&
            it.filters.lift == filters.lift &&
            it.filters.metric == variant.metric
    }

    val total = row?.summary?.total
    val totalDelta = when {
        variant.isCurrent && total != null -> 0L
        total != null && baseTotal != null -> total.toLong() - baseTotal.toLong()
        else -> null
    }

    return CohortComparisonRowPresentation(
        id = "wc=${variant.wc}|age=${variant.age}|tested=${variant.tested}|lift=${filters.lift}|metric=${variant.metric}",
        label = variant.label,
        wc = variant.wc,
        age = variant.age,
        tested = variant.tested,
        metric = variant.metric,
        total = total,
        totalDelta = totalDelta,
        minKg = row?.summary?.minKg,
        maxKg = row?.summary?.maxKg,
        status = when {
            row == null -> "slice missing"
            row.summary == null -> "summary missing"
            else -> "embedded summary"
        },
        statusOk = row?.summary != null,
        isCurrent = variant.isCurrent,
    )
}

private fun buildHomeState(
    snapshot: DatasetSnapshot,
    requestedFilters: LookupFilters,
    repository: PublishedDataRepository,
    loadMode: LookupPayloadLoadMode,
): HomeUiState {
    val sexes = sexOptions(snapshot.rootIndex)
    val sex = pickPreferredValue(sexes, requestedFilters.sex, "M")
    val equips = equipOptions(snapshot.rootIndex, sex)
    val equip = pickPreferredValue(equips, requestedFilters.equip, "Raw")
    val shardKey = "sex=$sex|equip=$equip"
    val shardPath = snapshot.rootIndex.shards[shardKey]

    if (shardPath == null) {
        return HomeUiState(
            isLoading = false,
            cachedLatestVersion = snapshot.latest.version,
            latest = snapshot.latest,
            rootShardCount = snapshot.rootIndex.shards.size,
            trendsPreview = snapshot.trendsPreview,
            loadSummary = buildLoadSummary(snapshot),
            errorMessage = "Root index did not contain shard $shardKey.",
        )
    }

    val sliceIndex = snapshot.sliceIndexByShard[shardKey] ?: repository
        .fetchSliceIndex(snapshot.latest.version, shardPath)
        .getOrElse { error ->
            return HomeUiState(
                isLoading = false,
                cachedLatestVersion = snapshot.latest.version,
                latest = snapshot.latest,
                rootShardCount = snapshot.rootIndex.shards.size,
                trendsPreview = snapshot.trendsPreview,
                loadSummary = buildLoadSummary(snapshot),
                errorMessage = error.message ?: "Failed to load shard index.",
            )
        }
        .also { loaded ->
            snapshot.sliceIndexByShard[shardKey] = loaded.value
            snapshot.sliceIndexSourceByShard[shardKey] = loaded.source
        }
        .value

    val slicePreview = buildSliceIndexPreview(
        shardKey = shardKey,
        shardPath = shardPath,
        sliceIndex = sliceIndex,
    )

    val rows = buildLookupRows(sliceIndex)
    if (rows.isEmpty()) {
        return HomeUiState(
            isLoading = false,
            cachedLatestVersion = snapshot.latest.version,
            latest = snapshot.latest,
            rootShardCount = snapshot.rootIndex.shards.size,
            shardPreview = slicePreview,
            trendsPreview = snapshot.trendsPreview,
            loadSummary = buildLoadSummary(snapshot, shardKey = shardKey),
            errorMessage = "Selected shard did not contain any slice rows.",
        )
    }

    val selectorState = buildSelectorState(
        rootIndex = snapshot.rootIndex,
        rows = rows,
        requestedFilters = requestedFilters.copy(
            sex = sex,
            equip = equip,
        ),
    )
    val trendSeries = snapshot.trendBucket?.let { bucket ->
        buildTrendSeriesPresentation(bucket, snapshot.trendSeries, selectorState.filters)
    }

    val row = currentRow(rows, selectorState.filters)
    if (row == null) {
        return HomeUiState(
            isLoading = false,
            cachedLatestVersion = snapshot.latest.version,
            latest = snapshot.latest,
            rootShardCount = snapshot.rootIndex.shards.size,
            shardPreview = slicePreview,
            selectorState = selectorState,
            trendsPreview = snapshot.trendsPreview,
            trendSeries = trendSeries,
            loadSummary = buildLoadSummary(snapshot, shardKey = shardKey),
            errorMessage = "Current filters did not resolve to an exact slice.",
        )
    }
    val comparisonRows = buildComparisonRows(
        rows = rows,
        filters = selectorState.filters,
        baseTotal = row.summary?.total,
    )

    if (loadMode == LookupPayloadLoadMode.SUMMARY_ONLY) {
        return HomeUiState(
            isLoading = false,
            cachedLatestVersion = snapshot.latest.version,
            latest = snapshot.latest,
            rootShardCount = snapshot.rootIndex.shards.size,
            shardPreview = slicePreview,
            selectorState = selectorState,
            trendsPreview = snapshot.trendsPreview,
            trendSeries = trendSeries,
            comparisonRows = comparisonRows,
            loadSummary = buildLoadSummary(snapshot, shardKey = shardKey),
            errorMessage = null,
        )
    }

    val histogram = snapshot.histogramByPath[row.histPath] ?: repository
        .fetchHistogram(snapshot.latest.version, row.histPath)
        .getOrElse { error ->
            return HomeUiState(
                isLoading = false,
                cachedLatestVersion = snapshot.latest.version,
                latest = snapshot.latest,
                rootShardCount = snapshot.rootIndex.shards.size,
                shardPreview = slicePreview,
                selectorState = selectorState,
                trendsPreview = snapshot.trendsPreview,
                trendSeries = trendSeries,
                comparisonRows = comparisonRows,
                loadSummary = buildLoadSummary(snapshot, shardKey = shardKey),
                errorMessage = error.message ?: "Failed to load histogram binary.",
            )
        }
        .also { loaded ->
            snapshot.histogramByPath[row.histPath] = loaded.value
            snapshot.histogramSourceByPath[row.histPath] = loaded.source
        }
        .value

    val heatmap = snapshot.heatmapByPath[row.heatPath] ?: repository
        .fetchHeatmap(snapshot.latest.version, row.heatPath)
        .getOrNull()
        ?.also { loaded ->
            snapshot.heatmapByPath[row.heatPath] = loaded.value
            snapshot.heatmapSourceByPath[row.heatPath] = loaded.source
        }
        ?.value

    val crossSexPreview = buildCrossSexLookupPresentation(
        snapshot = snapshot,
        repository = repository,
        filters = selectorState.filters,
        currentRow = row,
        currentHistogram = histogram,
    )

    return HomeUiState(
        isLoading = false,
        cachedLatestVersion = snapshot.latest.version,
        latest = snapshot.latest,
        rootShardCount = snapshot.rootIndex.shards.size,
        shardPreview = slicePreview,
        selectorState = selectorState,
        lookupPreview = buildHistogramLookupPreview(row, histogram, heatmap),
        crossSexPreview = crossSexPreview,
        trendsPreview = snapshot.trendsPreview,
        trendSeries = trendSeries,
        comparisonRows = comparisonRows,
        loadSummary = buildLoadSummary(
            snapshot = snapshot,
            shardKey = shardKey,
            histogramPath = row.histPath,
            heatmapPath = row.heatPath,
        ),
        errorMessage = null,
    )
}

private fun buildHistogramLookupPreview(
    row: LookupSliceRow,
    histogram: HistogramBin,
    heatmap: HeatmapBin?,
): HistogramLookupPreview {
    return HistogramLookupPreview(
        sliceKey = row.rawKey,
        cohortLabel = buildCohortLabel(row.filters),
        liftLabel = liftLabel(row.filters.lift),
        metric = row.filters.metric,
        histPath = row.histPath,
        heatPath = row.heatPath,
        histogram = histogram,
        heatmap = heatmap,
        p50 = valueForPercentile(histogram, 0.50f),
        p90 = valueForPercentile(histogram, 0.90f),
    )
}

private fun buildCohortLabel(filters: LookupFilters): String {
    return listOf(
        sexLabel(filters.sex),
        filters.equip,
        when (val wc = filters.wc) {
            "All" -> "All bodyweights"
            else -> "$wc kg class"
        },
        filters.age,
        when (filters.tested) {
            "Yes" -> "Tested"
            "All" -> "All tested statuses"
            else -> filters.tested
        },
    ).joinToString(" • ")
}

private fun buildCrossSexLookupPresentation(
    snapshot: DatasetSnapshot,
    repository: PublishedDataRepository,
    filters: LookupFilters,
    currentRow: LookupSliceRow,
    currentHistogram: HistogramBin,
): CrossSexLookupPresentation? {
    val male = loadCrossSexCohort(
        snapshot = snapshot,
        repository = repository,
        requestedFilters = filters,
        targetSex = "M",
        currentRow = currentRow,
        currentHistogram = currentHistogram,
    ) ?: return null
    val female = loadCrossSexCohort(
        snapshot = snapshot,
        repository = repository,
        requestedFilters = filters,
        targetSex = "F",
        currentRow = currentRow,
        currentHistogram = currentHistogram,
    ) ?: return null

    return CrossSexLookupPresentation(
        liftLabel = liftLabel(filters.lift),
        metric = filters.metric,
        male = male,
        female = female,
    )
}

private fun loadCrossSexCohort(
    snapshot: DatasetSnapshot,
    repository: PublishedDataRepository,
    requestedFilters: LookupFilters,
    targetSex: String,
    currentRow: LookupSliceRow,
    currentHistogram: HistogramBin,
): CrossSexCohortPresentation? {
    val match = if (requestedFilters.sex == targetSex) {
        CrossSexRowMatch(
            row = currentRow,
            note = null,
        )
    } else {
        resolveCrossSexRowMatch(
            snapshot = snapshot,
            repository = repository,
            requestedFilters = requestedFilters,
            targetSex = targetSex,
        )
    } ?: return null

    val histogram = if (match.row.histPath == currentRow.histPath) {
        currentHistogram
    } else {
        loadHistogramFromCacheOrRepository(
            snapshot = snapshot,
            repository = repository,
            histogramPath = match.row.histPath,
        ) ?: return null
    }

    return CrossSexCohortPresentation(
        label = buildCohortLabel(match.row.filters),
        histogram = histogram,
        note = match.note,
    )
}

private fun resolveCrossSexRowMatch(
    snapshot: DatasetSnapshot,
    repository: PublishedDataRepository,
    requestedFilters: LookupFilters,
    targetSex: String,
): CrossSexRowMatch? {
    val shardKey = "sex=$targetSex|equip=${requestedFilters.equip}"
    val shardPath = snapshot.rootIndex.shards[shardKey] ?: return null
    val sliceIndex = snapshot.sliceIndexByShard[shardKey] ?: repository
        .fetchSliceIndex(snapshot.latest.version, shardPath)
        .getOrNull()
        ?.also { loaded ->
            snapshot.sliceIndexByShard[shardKey] = loaded.value
            snapshot.sliceIndexSourceByShard[shardKey] = loaded.source
        }
        ?.value
        ?: return null

    val rows = buildLookupRows(sliceIndex)
    val requested = requestedFilters.copy(sex = targetSex)
    val variants = listOf(
        requested,
        requested.copy(wc = "All"),
        requested.copy(age = "All Ages"),
        requested.copy(tested = "All"),
        requested.copy(wc = "All", age = "All Ages"),
        requested.copy(wc = "All", age = "All Ages", tested = "All"),
    )

    val resolved = variants.firstNotNullOfOrNull { candidate ->
        currentRow(rows, candidate)?.let { row ->
            CrossSexRowMatch(
                row = row,
                note = buildCrossSexFallbackNote(requested = requested, resolved = row.filters),
            )
        }
    }

    return resolved
}

private fun buildCrossSexFallbackNote(
    requested: LookupFilters,
    resolved: LookupFilters,
): String? {
    val notes = mutableListOf<String>()
    if (resolved.wc != requested.wc) {
        notes += "bodyweight ${requested.wc} -> ${resolved.wc}"
    }
    if (resolved.age != requested.age) {
        notes += "age ${requested.age} -> ${resolved.age}"
    }
    if (resolved.tested != requested.tested) {
        notes += "tested ${requested.tested} -> ${resolved.tested}"
    }
    return if (notes.isEmpty()) null else "Fallback used: ${notes.joinToString(", ")}."
}

private fun loadHistogramFromCacheOrRepository(
    snapshot: DatasetSnapshot,
    repository: PublishedDataRepository,
    histogramPath: String,
): HistogramBin? {
    snapshot.histogramByPath[histogramPath]?.let { return it }
    val loaded = repository.fetchHistogram(snapshot.latest.version, histogramPath).getOrNull() ?: return null
    snapshot.histogramByPath[histogramPath] = loaded.value
    snapshot.histogramSourceByPath[histogramPath] = loaded.source
    return loaded.value
}

private fun buildLookupRows(sliceIndex: SliceIndex): List<LookupSliceRow> {
    val rows = when (val slices = sliceIndex.slices) {
        is SliceIndexEntries.MapEntries -> {
            slices.entries.mapNotNull { (rawKey, entry) ->
                parseLookupFilters(rawKey)?.let { filters ->
                    LookupSliceRow(
                        rawKey = rawKey,
                        filters = filters,
                        histPath = entry.hist,
                        heatPath = entry.heat,
                        summary = entry.summary,
                    )
                }
            }
        }

        is SliceIndexEntries.KeyEntries -> {
            slices.keys.mapNotNull { rawKey ->
                val filters = parseLookupFilters(rawKey) ?: return@mapNotNull null
                val histPath = histPathFromSliceKey(rawKey) ?: return@mapNotNull null
                val heatPath = heatPathFromSliceKey(rawKey) ?: return@mapNotNull null
                LookupSliceRow(
                    rawKey = rawKey,
                    filters = filters,
                    histPath = histPath,
                    heatPath = heatPath,
                    summary = null,
                )
            }
        }
    }

    val canonicalRows = LinkedHashMap<LookupFilters, LookupSliceRow>()
    for (row in rows) {
        val current = canonicalRows[row.filters]
        if (current == null || isPreferredLookupRowCandidate(candidate = row, current = current)) {
            canonicalRows[row.filters] = row
        }
    }

    return canonicalRows.values.sortedWith(
        compareBy<LookupSliceRow>(
            { ipfClassSortKey(it.filters.wc).first },
            { ipfClassSortKey(it.filters.wc).second },
            { ageClassSortKey(it.filters.age).first },
            { ageClassSortKey(it.filters.age).second },
            { it.filters.tested },
            { it.filters.lift },
            { it.filters.metric },
        ),
    )
}

private fun isPreferredLookupRowCandidate(
    candidate: LookupSliceRow,
    current: LookupSliceRow,
): Boolean {
    val candidateMetricExplicit =
        PublishedSliceContract.parseSliceKey(candidate.rawKey)?.metricExplicit == true
    val currentMetricExplicit =
        PublishedSliceContract.parseSliceKey(current.rawKey)?.metricExplicit == true
    if (candidateMetricExplicit != currentMetricExplicit) {
        return candidateMetricExplicit
    }

    val candidateHasSummary = candidate.summary != null
    val currentHasSummary = current.summary != null
    return candidateHasSummary && !currentHasSummary
}

private fun buildSelectorState(
    rootIndex: RootIndex,
    rows: List<LookupSliceRow>,
    requestedFilters: LookupFilters,
): LookupSelectorState {
    val sexes = sexOptions(rootIndex)
    val sex = pickPreferredValue(sexes, requestedFilters.sex, "M")
    val equips = equipOptions(rootIndex, sex)
    val equip = pickPreferredValue(equips, requestedFilters.equip, "Raw")

    val weightClasses = rows.map { it.filters.wc }.distinctWeightClasses()
    val wc = pickPreferredValue(weightClasses, requestedFilters.wc, "All")

    val ages = rows
        .filter { it.filters.wc == wc }
        .map { it.filters.age }
        .distinctAges()
    val age = pickPreferredValue(ages, requestedFilters.age, "All Ages")

    val testedOptions = rows
        .filter { it.filters.wc == wc && it.filters.age == age }
        .map { it.filters.tested }
        .distinctSorted()
    val tested = pickPreferredValue(testedOptions, requestedFilters.tested, "All")

    val liftOptions = rows
        .filter {
            it.filters.wc == wc &&
                it.filters.age == age &&
                it.filters.tested == tested
        }
        .map { it.filters.lift }
        .distinctSorted()
    val lift = pickPreferredValue(liftOptions, requestedFilters.lift, "T")

    val metricOptions = rows
        .filter {
            it.filters.wc == wc &&
                it.filters.age == age &&
                it.filters.tested == tested &&
                it.filters.lift == lift
        }
        .map { it.filters.metric }
        .distinctSorted()
    val metric = pickPreferredValue(metricOptions, requestedFilters.metric, "Kg")

    return LookupSelectorState(
        filters = LookupFilters(
            sex = sex,
            equip = equip,
            wc = wc,
            age = age,
            tested = tested,
            lift = lift,
            metric = metric,
        ),
        options = LookupFilterOptions(
            sexes = sexes,
            equips = equips,
            weightClasses = weightClasses,
            ages = ages,
            tested = testedOptions,
            lifts = liftOptions,
            metrics = metricOptions,
        ),
    )
}

private fun defaultFilters(
    rootIndex: RootIndex,
    profile: ProfileLookupDefaults = ProfileLookupDefaults(),
): LookupFilters {
    val preferredSex = profile.sex.ifEmpty { "M" }
    val preferredEquip = profile.equipment.ifEmpty { "Raw" }
    val preferredTested = profile.tested.ifEmpty { "All" }
    val sex = pickPreferredValue(sexOptions(rootIndex), "", preferredSex)
    val equip = pickPreferredValue(equipOptions(rootIndex, sex), "", preferredEquip)
    return LookupFilters(
        sex = sex,
        equip = equip,
        wc = "All",
        age = "All Ages",
        tested = preferredTested,
        lift = "T",
        metric = "Kg",
    )
}

private fun currentRow(
    rows: List<LookupSliceRow>,
    filters: LookupFilters,
): LookupSliceRow? {
    return rows.firstOrNull { row ->
        row.filters.wc == filters.wc &&
            row.filters.age == filters.age &&
            row.filters.tested == filters.tested &&
            row.filters.lift == filters.lift &&
            row.filters.metric == filters.metric
    }
}

private fun sexOptions(rootIndex: RootIndex): List<String> {
    return rootIndex.shards.keys
        .mapNotNull { PublishedSliceContract.parseShardKey(it)?.sex }
        .distinctSorted()
}

private fun equipOptions(
    rootIndex: RootIndex,
    sex: String,
): List<String> {
    return rootIndex.shards.keys
        .mapNotNull { key ->
            val shardKey = PublishedSliceContract.parseShardKey(key) ?: return@mapNotNull null
            if (shardKey.sex == sex) shardKey.equip else null
        }
        .distinctSorted()
}

private fun parseLookupFilters(raw: String): LookupFilters? {
    return PublishedSliceContract.parseSliceKey(raw)?.toLookupFilters()
}
private fun LookupFilters.updated(
    field: LookupFilterField,
    value: String,
): LookupFilters {
    return when (field) {
        LookupFilterField.SEX -> copy(sex = value)
        LookupFilterField.EQUIP -> copy(equip = value)
        LookupFilterField.WEIGHT_CLASS -> copy(wc = value)
        LookupFilterField.AGE -> copy(age = value)
        LookupFilterField.TESTED -> copy(tested = value)
        LookupFilterField.LIFT -> copy(lift = value)
        LookupFilterField.METRIC -> copy(metric = value)
    }
}

private fun pickPreferredValue(
    options: List<String>,
    current: String,
    preferred: String,
): String {
    if (options.isEmpty()) {
        return ""
    }
    if (current.isNotBlank() && current in options) {
        return current
    }
    if (preferred in options) {
        return preferred
    }
    return options.first()
}

private fun List<String>.distinctSorted(): List<String> = toSortedSet().toList()

private fun List<String>.distinctWeightClasses(): List<String> {
    return distinct().sortedWith(
        compareBy<String>(
            { ipfClassSortKey(it).first },
            { ipfClassSortKey(it).second },
        ),
    )
}

private fun List<String>.distinctAges(): List<String> {
    return distinct().sortedWith(
        compareBy<String>(
            { ageClassSortKey(it).first },
            { ageClassSortKey(it).second },
        ),
    )
}

private fun ipfClassSortKey(className: String): Pair<Int, Int> {
    if (className == "All") {
        return 0 to -1
    }
    val plusValue = className.removeSuffix("+").toIntOrNull()
    if (className.endsWith('+') && plusValue != null) {
        return 2 to plusValue
    }
    val numeric = className.toIntOrNull()
    if (numeric != null) {
        return 1 to numeric
    }
    return 3 to Int.MAX_VALUE
}

private fun ageClassSortKey(className: String): Pair<Int, Int> {
    if (className == "All Ages") {
        return 0 to -1
    }
    val start = className
        .split('-', '+')
        .firstOrNull()
        ?.toIntOrNull()
        ?: Int.MAX_VALUE
    return 1 to start
}

private fun sexLabel(raw: String?): String {
    return when (raw) {
        "M" -> "Men"
        "F" -> "Women"
        else -> raw ?: "Unknown sex"
    }
}

private fun liftLabel(raw: String?): String {
    return when (raw) {
        "S" -> "Squat"
        "B" -> "Bench"
        "D" -> "Deadlift"
        "T" -> "Total"
        else -> raw ?: "Lift"
    }
}

private fun histPathFromSliceKey(raw: String): String? {
    return PublishedSliceContract.histPathFromSliceKey(raw)
}

private fun heatPathFromSliceKey(raw: String): String? {
    return PublishedSliceContract.heatPathFromSliceKey(raw)
}

private fun PublishedSliceKey.toLookupFilters(): LookupFilters {
    return LookupFilters(
        sex = sex,
        equip = equip,
        wc = wc,
        age = age,
        tested = tested,
        lift = lift,
        metric = metric,
    )
}
