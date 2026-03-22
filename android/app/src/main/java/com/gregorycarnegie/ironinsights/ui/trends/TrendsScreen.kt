package com.gregorycarnegie.ironinsights.ui.trends

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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.model.TrendsPreview
import com.gregorycarnegie.ironinsights.ui.home.HomeUiState
import com.gregorycarnegie.ironinsights.ui.home.LoadSourcesCard
import com.gregorycarnegie.ironinsights.ui.home.LookupFilterField
import com.gregorycarnegie.ironinsights.ui.home.LookupSelectorState
import com.gregorycarnegie.ironinsights.ui.home.SectionCard
import com.gregorycarnegie.ironinsights.ui.home.SelectorChipRow
import com.gregorycarnegie.ironinsights.ui.home.TrendMetricCard
import com.gregorycarnegie.ironinsights.ui.home.TrendSeriesPresentation
import com.gregorycarnegie.ironinsights.ui.home.TrendSparkline
import com.gregorycarnegie.ironinsights.ui.home.ageOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.formatCount
import com.gregorycarnegie.ironinsights.ui.home.formatMetricValue
import com.gregorycarnegie.ironinsights.ui.home.liftOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.metricOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.sexOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.testedOptionLabel
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.navigation.AppRouteTabs
import java.util.Locale
import kotlin.math.max
import kotlin.math.min

@Composable
fun TrendsScreen(
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
            MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.7f),
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
                SectionCard(
                    title = "Trends over time",
                    eyebrow = "Yearly cohorts",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text(
                            text = "The Android client is using the same yearly trend payload as the website. Each series is keyed by sex, equipment, tested status, lift, and metric.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "Age and bodyweight class stay in the percentile lookup flow. The published trends aggregate across those buckets for the selected lift slice.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }

            uiState.selectorState?.let { selectorState ->
                item {
                    TrendSelectorCard(
                        selectorState = selectorState,
                        enabled = !uiState.isLoading,
                        onFilterChange = onFilterChange,
                    )
                }
            }

            uiState.trendSeries?.let { trends ->
                item {
                    TrendOverviewCard(
                        trends = trends,
                        axisLabel = buildThresholdAxisLabel(uiState),
                    )
                }
            } ?: item {
                TrendsUnavailableCard(uiState = uiState)
            }

            uiState.trendsPreview?.let { preview ->
                item {
                    TrendsPayloadCard(preview = preview)
                }
            }

            uiState.loadSummary?.let { summary ->
                item {
                    LoadSourcesCard(summary = summary)
                }
            }

            uiState.errorMessage?.let { error ->
                item {
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
            }

            item {
                Button(
                    onClick = onRefresh,
                    enabled = !uiState.isLoading,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Text(if (uiState.isLoading) "Loading..." else "Refresh trend payloads")
                }
            }
        }
    }
}

@Composable
private fun TrendSelectorCard(
    selectorState: LookupSelectorState,
    enabled: Boolean,
    onFilterChange: (LookupFilterField, String) -> Unit,
) {
    val filters = selectorState.filters
    val options = selectorState.options

    SectionCard(
        title = "Trend filters",
        eyebrow = "Series dimensions",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
            Text(
                text = "These selectors change the published yearly trend series. Weight class ${filters.wc} and age ${ageOptionLabel(filters.age)} stay lookup-only.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
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
private fun TrendOverviewCard(
    trends: TrendSeriesPresentation,
    axisLabel: String,
) {
    val latest = trends.points.lastOrNull() ?: return

    SectionCard(
        title = "Selected cohort",
        eyebrow = "Full trend view",
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
            ThresholdTrendCard(
                points = trends.points,
                axisLabel = axisLabel,
                p50Summary = trends.p50DriftSummary,
                p90Summary = trends.p90DriftSummary,
            )
        }
    }
}

@Composable
private fun ThresholdTrendCard(
    points: List<TrendPoint>,
    axisLabel: String,
    p50Summary: String,
    p90Summary: String,
) {
    if (points.size < 2) {
        return
    }

    val minValue = points.minOf { min(it.p50, it.p90) }
    val maxValue = points.maxOf { max(it.p50, it.p90) }
    val padding = ((maxValue - minValue) * 0.08f).coerceAtLeast(1f)
    val lower = minValue - padding
    val upper = maxValue + padding
    val valueSpan = (upper - lower).coerceAtLeast(1f)
    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.7f)
    val p50Color = MaterialTheme.colorScheme.primary
    val p90Color = MaterialTheme.colorScheme.tertiary

    fun scaledY(value: Float, chartHeight: Float): Float {
        return chartHeight - ((value - lower) / valueSpan) * chartHeight
    }

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Percentile thresholds by year",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "The website tracks yearly p50 and p90 thresholds for this cohort key. Android is rendering the same series here.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Column(
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                TrendLegendChip(
                    label = "p50",
                    color = p50Color,
                    summary = p50Summary,
                )
                TrendLegendChip(
                    label = "p90",
                    color = p90Color,
                    summary = p90Summary,
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = formatMetricValue(upper),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = axisLabel,
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(180.dp),
            ) {
                val chartWidth = size.width
                val chartHeight = size.height
                val stepX = chartWidth / points.lastIndex.toFloat()

                repeat(4) { index ->
                    val fraction = index / 3f
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
                        color = p50Color,
                        start = Offset((index - 1) * stepX, scaledY(previous.p50, chartHeight)),
                        end = Offset(index * stepX, scaledY(current.p50, chartHeight)),
                        strokeWidth = 4f,
                    )
                    drawLine(
                        color = p90Color,
                        start = Offset((index - 1) * stepX, scaledY(previous.p90, chartHeight)),
                        end = Offset(index * stepX, scaledY(current.p90, chartHeight)),
                        strokeWidth = 4f,
                    )
                }

                points.forEachIndexed { index, point ->
                    drawCircle(
                        color = p50Color,
                        radius = 4f,
                        center = Offset(index * stepX, scaledY(point.p50, chartHeight)),
                    )
                    drawCircle(
                        color = p90Color,
                        radius = 4f,
                        center = Offset(index * stepX, scaledY(point.p90, chartHeight)),
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
                    text = formatMetricValue(lower),
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
private fun TrendLegendChip(
    label: String,
    color: androidx.compose.ui.graphics.Color,
    summary: String,
) {
    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(12.dp)
                    .background(color = color, shape = RoundedCornerShape(99.dp)),
            )
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(
                    text = label,
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = summary,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun TrendsUnavailableCard(uiState: HomeUiState) {
    val message = when {
        uiState.isLoading -> "Loading yearly trend data for the selected cohort."
        uiState.trendsPreview == null -> "This dataset version did not expose a usable trends payload."
        else -> "No trend series matched the current filters. Try another lift, equipment class, or tested status."
    }

    SectionCard(
        title = "Trend availability",
        eyebrow = "Fallback",
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun TrendsPayloadCard(preview: TrendsPreview) {
    SectionCard(
        title = "Trend payload metadata",
        eyebrow = "Contract view",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(
                text = "Bucket: ${preview.bucket}",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "Series count: ${preview.seriesCount}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            preview.sampleSeriesKey?.let { key ->
                Text(
                    text = "Sample series: $key",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            preview.samplePointCount?.let { points ->
                Text(
                    text = "Sample points: $points",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Text(
                text = "Series key format: sex | equip | tested | lift | metric",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

private fun buildThresholdAxisLabel(uiState: HomeUiState): String {
    val preview = uiState.lookupPreview
    if (preview != null) {
        return "${preview.liftLabel} (${preview.metric.lowercase(Locale.US)})"
    }

    val filters = uiState.selectorState?.filters ?: return "Threshold value"
    return "${liftOptionLabel(filters.lift)} (${metricOptionLabel(filters.metric).lowercase(Locale.US)})"
}
