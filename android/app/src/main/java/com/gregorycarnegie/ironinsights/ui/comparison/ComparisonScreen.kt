package com.gregorycarnegie.ironinsights.ui.comparison

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.ui.home.CohortComparisonRowPresentation
import com.gregorycarnegie.ironinsights.ui.home.HomeUiState
import com.gregorycarnegie.ironinsights.ui.home.LoadSourcesCard
import com.gregorycarnegie.ironinsights.ui.home.LookupFilterField
import com.gregorycarnegie.ironinsights.ui.home.LookupFilters
import com.gregorycarnegie.ironinsights.ui.home.LookupSelectorCard
import com.gregorycarnegie.ironinsights.ui.home.SectionCard
import com.gregorycarnegie.ironinsights.ui.home.ageOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.formatCount
import com.gregorycarnegie.ironinsights.ui.home.formatMetricValue
import com.gregorycarnegie.ironinsights.ui.home.metricOptionLabel
import com.gregorycarnegie.ironinsights.ui.home.testedOptionLabel
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.navigation.AppRouteTabs
import java.util.Locale
import kotlin.math.abs

@Composable
fun ComparisonScreen(
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
            MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.72f),
        ),
    )

    val currentRow = uiState.comparisonRows.firstOrNull { it.isCurrent }
    val broaderRows = uiState.comparisonRows.filterNot { it.isCurrent }

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
                    title = "Cohort comparison",
                    eyebrow = "Embedded summaries",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text(
                            text = "These rows use embedded slice summaries only. They compare cohort size and observed range for nearby cohort variants.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "Exact percentile deltas and cross-sex comparisons are not loaded yet. This screen is the fast summary layer on top of the current shard index.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
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

            item {
                CurrentCohortCard(
                    filters = uiState.selectorState?.filters,
                    row = currentRow,
                )
            }

            item {
                BroaderCohortsCard(rows = broaderRows)
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
                    Text(if (uiState.isLoading) "Loading..." else "Refresh comparison data")
                }
            }
        }
    }
}

@Composable
private fun CurrentCohortCard(
    filters: LookupFilters?,
    row: CohortComparisonRowPresentation?,
) {
    val cohortLabel = filters?.let(::fallbackCohortLabel)
    val total = row?.total?.toLong()
    val minValue = row?.minKg
    val maxValue = row?.maxKg
    val metric = row?.metric ?: filters?.metric ?: "Kg"

    SectionCard(
        title = "Current cohort",
        eyebrow = "Baseline",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
            cohortLabel?.let { label ->
                Text(
                    text = label,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Lifters",
                    value = total?.let { formatCount(it) } ?: "n/a",
                    subtitle = "Current slice cohort size.",
                )
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Observed range",
                    value = formatObservedRange(minValue, maxValue, metric),
                    subtitle = observedRangeSubtitle(metric),
                )
            }
            Text(
                text = row?.status ?: "Current slice summary still loading.",
                style = MaterialTheme.typography.bodySmall,
                color = if (row?.statusOk == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun BroaderCohortsCard(rows: List<CohortComparisonRowPresentation>) {
    SectionCard(
        title = "Broader cohorts",
        eyebrow = "Quick rows",
    ) {
        if (rows.isEmpty()) {
            Text(
                text = "No comparison rows are available for the current selection yet.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                rows.forEach { row ->
                    ComparisonRowCard(row = row)
                }
            }
        }
    }
}

@Composable
private fun ComparisonRowCard(row: CohortComparisonRowPresentation) {
    val containerColor = when {
        row.statusOk -> MaterialTheme.colorScheme.surface.copy(alpha = 0.94f)
        else -> MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.82f)
    }

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = containerColor,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = row.label,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "wc ${row.wc} • ${ageOptionLabel(row.age)} • ${testedOptionLabel(row.tested)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Lifters",
                    value = row.total?.let { formatCount(it.toLong()) } ?: "n/a",
                    subtitle = "Embedded cohort size.",
                )
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Δ lifters",
                    value = formatDelta(row.totalDelta),
                    subtitle = "Vs current slice.",
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Observed range",
                    value = formatObservedRange(row.minKg, row.maxKg, row.metric),
                    subtitle = observedRangeSubtitle(row.metric),
                )
                StatusPill(
                    status = row.status,
                    ok = row.statusOk,
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Composable
private fun ComparisonStatCard(
    modifier: Modifier = Modifier,
    title: String,
    value: String,
    subtitle: String,
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.68f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.86f),
            )
        }
    }
}

@Composable
private fun StatusPill(
    status: String,
    ok: Boolean,
    modifier: Modifier = Modifier,
) {
    val background = if (ok) {
        MaterialTheme.colorScheme.secondaryContainer
    } else {
        MaterialTheme.colorScheme.surfaceVariant
    }
    val foreground = if (ok) {
        MaterialTheme.colorScheme.onSecondaryContainer
    } else {
        MaterialTheme.colorScheme.onSurfaceVariant
    }

    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(999.dp),
        color = background,
    ) {
        Text(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
            text = status,
            style = MaterialTheme.typography.labelLarge,
            color = foreground,
        )
    }
}

private fun fallbackCohortLabel(filters: LookupFilters): String {
    return listOf(
        when (filters.sex) {
            "M" -> "Men"
            "F" -> "Women"
            else -> filters.sex
        },
        filters.equip,
        if (filters.wc == "All") "All bodyweights" else "${filters.wc} kg class",
        ageOptionLabel(filters.age),
        testedOptionLabel(filters.tested),
    ).joinToString(" • ")
}

private fun formatObservedRange(
    minKg: Float?,
    maxKg: Float?,
    metric: String,
): String {
    return if (minKg != null && maxKg != null) {
        "${formatMetricValue(minKg)}-${formatMetricValue(maxKg)} ${metric.lowercase(Locale.US)}"
    } else {
        "n/a"
    }
}

private fun observedRangeSubtitle(metric: String): String {
    return "Published min-max ${metricOptionLabel(metric).lowercase(Locale.US)}."
}

private fun formatDelta(delta: Long?): String {
    return when {
        delta == null -> "n/a"
        delta == 0L -> "0"
        delta > 0L -> "+${formatCount(delta)}"
        else -> "-${formatCount(abs(delta))}"
    }
}
