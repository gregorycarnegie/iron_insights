package com.gregorycarnegie.ironinsights.ui.home

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.config.AppConfig
import com.gregorycarnegie.ironinsights.config.EnvironmentConfig
import com.gregorycarnegie.ironinsights.data.model.BodyweightConditionedLookup
import com.gregorycarnegie.ironinsights.data.model.DatasetEndpoint
import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSource
import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSummary
import com.gregorycarnegie.ironinsights.data.model.HistogramLookupPreview
import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.repository.bodyweightConditionedPercentile
import com.gregorycarnegie.ironinsights.data.repository.percentileForValue
import java.text.NumberFormat
import java.util.Locale
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.navigation.AppRouteTabs

@Composable
fun HomeScreen(
    environment: EnvironmentConfig,
    endpoints: List<DatasetEndpoint>,
    milestones: List<String>,
    uiState: HomeUiState,
    selectedRoute: AppRoute,
    onRouteChange: (AppRoute) -> Unit,
    onRefresh: () -> Unit,
    onFilterChange: (LookupFilterField, String) -> Unit,
) {
    val background = Brush.verticalGradient(
        colors = listOf(
            MaterialTheme.colorScheme.background,
            MaterialTheme.colorScheme.surface,
            MaterialTheme.colorScheme.primaryContainer,
        ),
    )

    Scaffold(containerColor = MaterialTheme.colorScheme.background) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .background(background),
            contentPadding = PaddingValues(
                start = 20.dp,
                top = innerPadding.calculateTopPadding() + 20.dp,
                end = 20.dp,
                bottom = innerPadding.calculateBottomPadding() + 28.dp,
            ),
            verticalArrangement = Arrangement.spacedBy(18.dp),
        ) {
            item {
                AppRouteTabs(
                    selectedRoute = selectedRoute,
                    onRouteChange = onRouteChange,
                )
            }
            item {
                HeroCard(environment = environment)
            }

            item {
                StatusCard(
                    environment = environment,
                    uiState = uiState,
                    onRefresh = onRefresh,
                )
            }

            uiState.selectorState?.let { selectorState ->
                item {
                    LookupSelectorCard(
                        selectorState = selectorState,
                        enabled = !uiState.isLoading,
                        onFilterChange = onFilterChange,
                    )
                }
            }

            uiState.lookupPreview?.let { lookupPreview ->
                item {
                    PercentileLookupCard(preview = lookupPreview)
                }
            }

            uiState.trendSeries?.let { trendSeries ->
                item {
                    TrendsCard(trends = trendSeries)
                }
            }

            item {
                SectionCard(
                    title = "Dataset contract",
                    eyebrow = "Reuse the wheel",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
                        endpoints.forEachIndexed { index, endpoint ->
                            EndpointRow(endpoint = endpoint)
                            if (index < endpoints.lastIndex) {
                                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                            }
                        }
                    }
                }
            }

            item {
                SectionCard(
                    title = "Immediate milestones",
                    eyebrow = "What happens next",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        milestones.forEach { milestone ->
                            MilestoneRow(text = milestone)
                        }
                    }
                }
            }
        }
    }
}

@Composable
internal fun LookupSelectorCard(
    selectorState: LookupSelectorState,
    enabled: Boolean,
    onFilterChange: (LookupFilterField, String) -> Unit,
) {
    val filters = selectorState.filters
    val options = selectorState.options

    SectionCard(
        title = "Lookup filters",
        eyebrow = "Choose cohort",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
            SelectorChipRow(
                title = "Sex",
                options = options.sexes,
                selected = filters.sex,
                enabled = enabled,
                labelFor = ::sexOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.SEX, it) },
            )
            SelectorChipRow(
                title = "Equipment",
                options = options.equips,
                selected = filters.equip,
                enabled = enabled,
                labelFor = { it },
                onSelect = { onFilterChange(LookupFilterField.EQUIP, it) },
            )
            SelectorChipRow(
                title = "Bodyweight class",
                options = options.weightClasses,
                selected = filters.wc,
                enabled = enabled,
                labelFor = { if (it == "All") "All classes" else it },
                onSelect = { onFilterChange(LookupFilterField.WEIGHT_CLASS, it) },
            )
            SelectorChipRow(
                title = "Age",
                options = options.ages,
                selected = filters.age,
                enabled = enabled,
                labelFor = ::ageOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.AGE, it) },
            )
            SelectorChipRow(
                title = "Tested",
                options = options.tested,
                selected = filters.tested,
                enabled = enabled,
                labelFor = ::testedOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.TESTED, it) },
            )
            SelectorChipRow(
                title = "Lift",
                options = options.lifts,
                selected = filters.lift,
                enabled = enabled,
                labelFor = ::liftOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.LIFT, it) },
            )
            SelectorChipRow(
                title = "Metric",
                options = options.metrics,
                selected = filters.metric,
                enabled = enabled,
                labelFor = ::metricOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.METRIC, it) },
            )
        }
    }
}

@Composable
internal fun SelectorChipRow(
    title: String,
    options: List<String>,
    selected: String,
    enabled: Boolean,
    labelFor: (String) -> String,
    onSelect: (String) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(options, key = { it }) { option ->
                FilterChip(
                    selected = option == selected,
                    onClick = { onSelect(option) },
                    enabled = enabled,
                    label = {
                        Text(labelFor(option))
                    },
                )
            }
        }
    }
}

@Composable
private fun PercentileLookupCard(preview: HistogramLookupPreview) {
    var liftInput by rememberSaveable(preview.sliceKey) {
        mutableStateOf(preview.p50?.let(::formatMetricValue).orEmpty())
    }
    var bodyweightInput by rememberSaveable(preview.sliceKey) {
        mutableStateOf("")
    }
    val parsedLift = liftInput.toFloatOrNull()
    val parsedBodyweight = bodyweightInput.toFloatOrNull()
    val lookup = parsedLift?.let { percentileForValue(preview.histogram, it) }
    val conditionedLookup = if (parsedLift != null && parsedBodyweight != null) {
        bodyweightConditionedPercentile(preview.heatmap, parsedLift, parsedBodyweight)
    } else {
        null
    }
    val topPercent = lookup?.let { ((1f - it.percentile).coerceAtLeast(0f) * 100f) }

    SectionCard(
        title = "Percentile lookup",
        eyebrow = "Live histogram",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(
                text = preview.cohortLabel,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "${preview.liftLabel} in ${preview.metric} from ${formatCount(preview.histogram.total)} lifters",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Surface(
                shape = RoundedCornerShape(18.dp),
                color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.72f),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = "Range ${formatMetricValue(preview.histogram.min)}-${formatMetricValue(preview.histogram.max)} ${preview.metric.lowercase(Locale.US)}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                    Text(
                        text = "Bin width ${formatMetricValue(preview.histogram.baseBin)} ${preview.metric.lowercase(Locale.US)}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                    Text(
                        text = "Median ${preview.p50?.let(::formatMetricValue) ?: "n/a"} • P90 ${preview.p90?.let(::formatMetricValue) ?: "n/a"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                }
            }
            OutlinedTextField(
                value = liftInput,
                onValueChange = { liftInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Enter ${preview.liftLabel.lowercase(Locale.US)} in ${preview.metric}") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            )
            when {
                liftInput.isBlank() -> {
                    Text(
                        text = "Enter a value to estimate percentile with the published histogram.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                parsedLift == null -> {
                    Text(
                        text = "Enter a valid number such as 500 or 182.5.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }

                lookup != null -> {
                    Surface(
                        shape = RoundedCornerShape(18.dp),
                        color = MaterialTheme.colorScheme.secondaryContainer,
                    ) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            Text(
                                text = "${formatPercent(lookup.percentile * 100f)} percentile",
                                style = MaterialTheme.typography.headlineMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                            Text(
                                text = "Approx rank ${formatCount(lookup.rank)} of ${formatCount(lookup.total)}",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                            Text(
                                text = "Top ${topPercent?.let(::formatPercent) ?: "n/a"} • Bin ${formatMetricValue(lookup.binLow)}-${formatMetricValue(lookup.binHigh)} ${preview.metric.lowercase(Locale.US)}",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                        }
                    }
                }
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Text(
                text = "Bodyweight-conditioned percentile",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            OutlinedTextField(
                value = bodyweightInput,
                onValueChange = { bodyweightInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Enter bodyweight in kg") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            )
            when {
                preview.heatmap == null -> {
                    Text(
                        text = "This metric appears after the selected slice heatmap loads.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                bodyweightInput.isBlank() -> {
                    Text(
                        text = "Enter bodyweight in kg to compare against nearby bodyweights.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                parsedBodyweight == null -> {
                    Text(
                        text = "Enter a valid bodyweight in kg such as 90 or 82.5.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }

                conditionedLookup != null -> {
                    BodyweightConditionedCard(
                        preview = preview,
                        lookup = conditionedLookup,
                    )
                }

                else -> {
                    Text(
                        text = "Bodyweight-conditioned metric unavailable for this heatmap.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
            Text(
                text = "Source slice: ${preview.sliceKey}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "Histogram file: ${preview.histPath}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "Heatmap file: ${preview.heatPath}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun BodyweightConditionedCard(
    preview: HistogramLookupPreview,
    lookup: BodyweightConditionedLookup,
) {
    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.tertiaryContainer,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = "Among nearby bodyweights (${formatMetricValue(lookup.bwWindowLow)}-${formatMetricValue(lookup.bwWindowHigh)} kg), you're stronger than ${formatPercent(lookup.percentile * 100f)}.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Text(
                text = "Rank ~${formatCount(lookup.rank)} / ${formatCount(lookup.totalNearby)} nearby lifters",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Text(
                text = "Current BW bin ${formatMetricValue(lookup.bwBinLow)}-${formatMetricValue(lookup.bwBinHigh)} kg",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Text(
                text = "Current heat cell ${formatCount(lookup.localCellCount)} • 3x3 neighborhood ${formatCount(lookup.neighborhoodCount)}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Text(
                text = "Neighborhood share ${formatPercent(lookup.neighborhoodShare * 100f)} • Lift bin ${formatMetricValue(lookup.liftBinLow)}-${formatMetricValue(lookup.liftBinHigh)} ${preview.metric.lowercase(Locale.US)}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
        }
    }
}

@Composable
private fun TrendsCard(trends: TrendSeriesPresentation) {
    val latest = trends.points.lastOrNull() ?: return

    SectionCard(
        title = "Trend snapshot",
        eyebrow = "Yearly cohort",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(
                text = trends.note,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            LazyRow(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                item {
                    TrendMetricCard(
                        title = "Latest year",
                        value = latest.year.toString(),
                        subtitle = trends.bucket.replaceFirstChar { it.uppercase() } + " buckets",
                    )
                }
                item {
                    TrendMetricCard(
                        title = "Cohort size",
                        value = formatCount(latest.total.toLong()),
                        subtitle = trends.growthSummary,
                    )
                }
                item {
                    TrendMetricCard(
                        title = "p50",
                        value = formatMetricValue(latest.p50),
                        subtitle = trends.p50DriftSummary,
                    )
                }
                item {
                    TrendMetricCard(
                        title = "p90",
                        value = formatMetricValue(latest.p90),
                        subtitle = trends.p90DriftSummary,
                    )
                }
            }
            TrendSparkline(points = trends.points)
        }
    }
}

@Composable
internal fun TrendMetricCard(
    title: String,
    value: String,
    subtitle: String,
) {
    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.secondaryContainer,
    ) {
        Column(
            modifier = Modifier
                .width(220.dp)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSecondaryContainer,
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSecondaryContainer,
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSecondaryContainer.copy(alpha = 0.84f),
            )
        }
    }
}

@Composable
internal fun TrendSparkline(points: List<TrendPoint>) {
    if (points.size < 2) {
        return
    }

    val minCount = points.minOf { it.total.coerceAtLeast(1).toFloat() }
    val maxCount = points.maxOf { it.total.coerceAtLeast(1).toFloat() }
    val safeMax = if (maxCount <= minCount) minCount + 1f else maxCount
    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.7f)
    val lineColor = MaterialTheme.colorScheme.primary

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = "Cohort size by year",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "Log scale keeps sparse early years visible beside recent larger cohorts.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = formatCount(safeMax.toLong()),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "Total lifters",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(148.dp),
            ) {
                val chartWidth = size.width
                val chartHeight = size.height
                val stepX = chartWidth / points.lastIndex.toFloat()
                val minLog = kotlin.math.log10(minCount.toDouble()).toFloat()
                val maxLog = kotlin.math.log10(safeMax.toDouble()).toFloat()
                val span = (maxLog - minLog).coerceAtLeast(0.1f)

                fun scaledY(value: Float): Float {
                    val currentLog = kotlin.math.log10(value.coerceAtLeast(1f).toDouble()).toFloat()
                    return chartHeight - ((currentLog - minLog) / span) * chartHeight
                }

                repeat(3) { index ->
                    val fraction = index / 2f
                    val y = chartHeight * fraction
                    drawLine(
                        color = gridColor,
                        start = Offset(0f, y),
                        end = Offset(chartWidth, y),
                        strokeWidth = 1.5f,
                    )
                }

                for (index in 1 until points.size) {
                    val previous = points[index - 1]
                    val current = points[index]
                    drawLine(
                        color = lineColor,
                        start = Offset((index - 1) * stepX, scaledY(previous.total.toFloat())),
                        end = Offset(index * stepX, scaledY(current.total.toFloat())),
                        strokeWidth = 5f,
                    )
                }

                points.forEachIndexed { index, point ->
                    drawCircle(
                        color = lineColor,
                        radius = 5f,
                        center = Offset(index * stepX, scaledY(point.total.toFloat())),
                    )
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = points.first().year.toString(),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = points.last().year.toString(),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
@Composable
private fun HeroCard(environment: EnvironmentConfig) {

    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
        ),
        shape = RoundedCornerShape(28.dp),
    ) {
        Column(
            modifier = Modifier.padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text(
                text = "Iron Insights",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = "Native Android shell wired to the live website dataset.",
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "The app will read the same published bundle as the site, starting from latest.json and walking into versioned indexes and binary slices.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Surface(
                shape = RoundedCornerShape(20.dp),
                color = MaterialTheme.colorScheme.secondaryContainer,
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = "Base URL",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = environment.siteBaseUrl,
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = AppConfig.resolvePublishedPath("data/latest.json"),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer.copy(alpha = 0.85f),
                    )
                }
            }
        }
    }
}

@Composable
private fun StatusCard(
    environment: EnvironmentConfig,
    uiState: HomeUiState,
    onRefresh: () -> Unit,
) {
    SectionCard(
        title = "Live dataset status",
        eyebrow = "First real fetch",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            uiState.cachedLatestVersion?.let { cachedVersion ->
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.tertiaryContainer,
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text(
                            text = "Cached latest version",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onTertiaryContainer,
                        )
                        Text(
                            text = cachedVersion,
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onTertiaryContainer,
                        )
                    }
                }
            }

            Surface(
                shape = RoundedCornerShape(18.dp),
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = "Request path",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Text(
                        text = environment.dataBaseUrl + "latest.json",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            when {
                uiState.isLoading && uiState.latest == null -> {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        CircularProgressIndicator(
                            modifier = Modifier.width(20.dp),
                            strokeWidth = 2.dp,
                        )
                        Text(
                            text = "Loading the active published dataset pointer.",
                            style = MaterialTheme.typography.bodyLarge,
                        )
                    }
                }

                uiState.latest != null -> {
                    Surface(
                        shape = RoundedCornerShape(18.dp),
                        color = MaterialTheme.colorScheme.secondaryContainer,
                    ) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Text(
                                text = "Latest dataset",
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                            Text(
                                text = uiState.latest.version,
                                style = MaterialTheme.typography.headlineMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                            uiState.rootShardCount?.let { shardCount ->
                                Text(
                                    text = "Root index shards: $shardCount",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onSecondaryContainer,
                                )
                            }
                            Text(
                                text = uiState.latest.revision ?: "No revision metadata published.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                        }
                    }
                }
            }

            uiState.shardPreview?.let { preview ->
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.primaryContainer,
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text(
                            text = "Current shard",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        Text(
                            text = preview.shardKey,
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        Text(
                            text = "Shard file: ${preview.shardPath}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        Text(
                            text = "Slice entries: ${preview.sliceCount}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        preview.sampleSliceKey?.let { sampleSliceKey ->
                            Text(
                                text = "Sample slice: $sampleSliceKey",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                            )
                        }
                        preview.sampleHistPath?.let { sampleHistPath ->
                            Text(
                                text = "Sample hist: $sampleHistPath",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                            )
                        }
                    }
                }
            }

            uiState.trendsPreview?.let { trends ->
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.surface.copy(alpha = 0.95f),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text(
                            text = "Trends payload",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "Bucket: ${trends.bucket}",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "Series count: ${trends.seriesCount}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        trends.sampleSeriesKey?.let { sampleSeriesKey ->
                            Text(
                                text = "Sample series: $sampleSeriesKey",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        trends.samplePointCount?.let { samplePointCount ->
                            Text(
                                text = "Sample points: $samplePointCount",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }

            uiState.loadSummary?.let { summary ->
                LoadSourcesCard(summary = summary)
            }

            uiState.errorMessage?.let { error ->
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.errorContainer,
                ) {
                    Text(
                        modifier = Modifier.padding(16.dp),
                        text = error,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onErrorContainer,
                    )
                }
            }

            Button(
                onClick = onRefresh,
                enabled = !uiState.isLoading,
            ) {
                Text(if (uiState.isLoading) "Loading..." else "Refresh latest.json")
            }
        }
    }
}

@Composable
internal fun LoadSourcesCard(summary: DatasetLoadSummary) {
    val sources = listOf(
        "Latest pointer" to summary.latest,
        "Root index" to summary.rootIndex,
        "Shard index" to summary.shardIndex,
        "Histogram" to summary.histogram,
        "Heatmap" to summary.heatmap,
        "Trends" to summary.trends,
    ).filter { (_, source) -> source != null }

    if (sources.isEmpty()) {
        return
    }

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.78f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = "Payload sources",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (summary.usesOfflineFallback()) {
                Text(
                    text = "Using cached payloads where the latest network request was unavailable.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
            sources.forEach { (label, source) ->
                val resolvedSource = source ?: return@forEach
                Text(
                    text = "$label: ${formatLoadSource(resolvedSource)}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
internal fun SectionCard(
    title: String,
    eyebrow: String,
    content: @Composable () -> Unit,
) {
    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
        ),
        shape = RoundedCornerShape(24.dp),
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = eyebrow,
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.tertiary,
                )
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold,
                )
            }
            content()
        }
    }
}

@Composable
private fun EndpointRow(endpoint: DatasetEndpoint) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            text = endpoint.title,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
        )
        Text(
            text = endpoint.relativePath,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.primary,
        )
        Text(
            text = endpoint.purpose,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun MilestoneRow(text: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(
            modifier = Modifier
                .padding(top = 6.dp)
                .width(10.dp)
                .height(10.dp)
                .background(
                    color = MaterialTheme.colorScheme.primary,
                    shape = RoundedCornerShape(99.dp),
                )
        )
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

internal fun formatMetricValue(value: Float): String {
    return if ((value * 10f).toInt() % 10 == 0) {
        String.format(Locale.US, "%.0f", value)
    } else {
        String.format(Locale.US, "%.1f", value)
    }
}

private fun formatPercent(value: Float): String {
    return String.format(Locale.US, "%.1f%%", value)
}

internal fun formatCount(value: Long): String {
    return NumberFormat.getIntegerInstance(Locale.US).format(value)
}

private fun formatLoadSource(source: DatasetLoadSource): String {
    return when (source) {
        DatasetLoadSource.NETWORK -> "Network"
        DatasetLoadSource.DISK_CACHE -> "Disk cache"
        DatasetLoadSource.VERSION_CACHE -> "Cached version pointer"
    }
}

internal fun sexOptionLabel(value: String): String {
    return when (value) {
        "M" -> "Men"
        "F" -> "Women"
        else -> value
    }
}

internal fun testedOptionLabel(value: String): String {
    return when (value) {
        "Yes" -> "Tested"
        else -> value
    }
}

internal fun liftOptionLabel(value: String): String {
    return when (value) {
        "S" -> "Squat"
        "B" -> "Bench"
        "D" -> "Deadlift"
        "T" -> "Total"
        else -> value
    }
}

internal fun metricOptionLabel(value: String): String {
    return when (value) {
        "Dots" -> "DOTS"
        else -> value
    }
}

internal fun ageOptionLabel(value: String): String {
    return when (value) {
        "All Ages" -> "All Ages"
        "5-12" -> "Youth 5-12"
        "13-15" -> "Teen 13-15"
        "16-17" -> "Teen 16-17"
        "18-19" -> "Teen 18-19"
        "20-23" -> "Juniors 20-23"
        "24-34" -> "Seniors 24-34"
        "35-39" -> "Submasters 35-39"
        else -> {
            if ('-' in value) {
                "Masters $value"
            } else if (value.endsWith('+')) {
                "Masters $value"
            } else {
                value
            }
        }
    }
}
