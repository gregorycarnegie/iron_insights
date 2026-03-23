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
import androidx.compose.foundation.BorderStroke
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
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
    var devExpanded by rememberSaveable { mutableStateOf(false) }

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
            verticalArrangement = Arrangement.spacedBy(22.dp),
        ) {
            item {
                AppRouteTabs(
                    selectedRoute = selectedRoute,
                    onRouteChange = onRouteChange,
                )
            }

            item {
                CurrentCohortCard(
                    filters = uiState.selectorState?.filters,
                    row = currentRow,
                )
            }

            if (broaderRows.isNotEmpty()) {
                item {
                    BroaderCohortsCard(rows = broaderRows)
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
                ComparisonDevInfo(
                    expanded = devExpanded,
                    onToggle = { devExpanded = !devExpanded },
                    uiState = uiState,
                    onRefresh = onRefresh,
                )
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
        title = "Your cohort",
        eyebrow = "Current selection",
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
                    subtitle = "In this cohort.",
                )
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Range",
                    value = formatObservedRange(minValue, maxValue, metric),
                    subtitle = observedRangeSubtitle(metric),
                )
            }
        }
    }
}

@Composable
private fun BroaderCohortsCard(rows: List<CohortComparisonRowPresentation>) {
    SectionCard(
        title = "Nearby cohorts",
        eyebrow = "How similar groups compare",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            rows.forEach { row ->
                ComparisonRowCard(row = row)
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
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Lifters",
                    value = row.total?.let { formatCount(it.toLong()) } ?: "n/a",
                    subtitle = formatDeltaLabel(row.totalDelta),
                )
                ComparisonStatCard(
                    modifier = Modifier.weight(1f),
                    title = "Range",
                    value = formatObservedRange(row.minKg, row.maxKg, row.metric),
                    subtitle = observedRangeSubtitle(row.metric),
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
private fun ComparisonDevInfo(
    expanded: Boolean,
    onToggle: () -> Unit,
    uiState: HomeUiState,
    onRefresh: () -> Unit,
) {
    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
        ),
        shape = RoundedCornerShape(20.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = "Data sources",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                FilterChip(
                    selected = expanded,
                    onClick = onToggle,
                    label = { Text(if (expanded) "Hide" else "Show") },
                )
            }
            if (expanded) {
                Column(
                    modifier = Modifier.padding(top = 14.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    uiState.loadSummary?.let { summary ->
                        LoadSourcesCard(summary = summary)
                    }
                    Button(
                        onClick = onRefresh,
                        enabled = !uiState.isLoading,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Text(if (uiState.isLoading) "Loading..." else "Refresh data")
                    }
                }
            }
        }
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
    ).joinToString(" · ")
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
    return "Min-max ${metricOptionLabel(metric).lowercase(Locale.US)}."
}

private fun formatDelta(delta: Long?): String {
    return when {
        delta == null -> "n/a"
        delta == 0L -> "0"
        delta > 0L -> "+${formatCount(delta)}"
        else -> "-${formatCount(abs(delta))}"
    }
}

private fun formatDeltaLabel(delta: Long?): String {
    return when {
        delta == null -> ""
        delta == 0L -> "Same as your cohort."
        delta > 0L -> "${formatDelta(delta)} more lifters."
        else -> "${formatDelta(delta)} fewer lifters."
    }
}
