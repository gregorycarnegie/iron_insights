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
import androidx.compose.foundation.BorderStroke
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
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
import android.content.Intent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.platform.LocalContext
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
    var filtersExpanded by rememberSaveable { mutableStateOf(false) }
    var devInfoExpanded by rememberSaveable { mutableStateOf(false) }

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
            verticalArrangement = Arrangement.spacedBy(22.dp),
        ) {
            item {
                AppRouteTabs(
                    selectedRoute = selectedRoute,
                    onRouteChange = onRouteChange,
                )
            }

            item {
                WelcomeHeader(
                    uiState = uiState,
                    isLoading = uiState.isLoading,
                )
            }

            uiState.selectorState?.let { selectorState ->
                item {
                    FilterSummaryBar(
                        selectorState = selectorState,
                        expanded = filtersExpanded,
                        onToggle = { filtersExpanded = !filtersExpanded },
                        enabled = !uiState.isLoading,
                        onFilterChange = onFilterChange,
                    )
                }
                if (selectorState.options.lifts.size > 1) {
                    item {
                        QuickLiftFocusCard(
                            selectorState = selectorState,
                            enabled = !uiState.isLoading,
                            onFilterChange = onFilterChange,
                        )
                    }
                }
            }

            uiState.lookupPreview?.let { lookupPreview ->
                item {
                    PercentileLookupCard(
                        preview = lookupPreview,
                        crossSexPreview = uiState.crossSexPreview,
                    )
                }
            }

            uiState.trendSeries?.let { trendSeries ->
                item {
                    TrendsCard(trends = trendSeries)
                }
            }

            if (uiState.lookupPreview != null) {
                item {
                    JourneyLinks(onRouteChange = onRouteChange)
                }
            }

            item {
                DevInfoSection(
                    expanded = devInfoExpanded,
                    onToggle = { devInfoExpanded = !devInfoExpanded },
                    environment = environment,
                    uiState = uiState,
                    endpoints = endpoints,
                    milestones = milestones,
                    onRefresh = onRefresh,
                )
            }
        }
    }
}

@Composable
private fun WelcomeHeader(
    uiState: HomeUiState,
    isLoading: Boolean,
) {
    val filters = uiState.selectorState?.filters
    val preview = uiState.lookupPreview
    val cohortSize = preview?.histogram?.total

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            text = "How strong are you?",
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurface,
        )
        when {
            isLoading && preview == null -> {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.width(18.dp),
                        strokeWidth = 2.dp,
                    )
                    Text(
                        text = "Loading powerlifting data...",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            filters != null && cohortSize != null -> {
                Text(
                    text = "Enter a lift below to see your percentile among ${formatCount(cohortSize)} lifters.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            filters != null -> {
                Text(
                    text = "Pick your filters, then enter a lift to see where you rank.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        uiState.errorMessage?.let { error ->
            Surface(
                shape = RoundedCornerShape(14.dp),
                color = MaterialTheme.colorScheme.errorContainer,
            ) {
                Text(
                    modifier = Modifier.padding(12.dp),
                    text = error,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onErrorContainer,
                )
            }
        }
    }
}

@Composable
private fun QuickLiftFocusCard(
    selectorState: LookupSelectorState,
    enabled: Boolean,
    onFilterChange: (LookupFilterField, String) -> Unit,
) {
    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = "Lift focus",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            SelectorChipRow(
                title = "Choose squat, bench, deadlift, or total",
                options = selectorState.options.lifts,
                selected = selectorState.filters.lift,
                enabled = enabled,
                labelFor = ::liftOptionLabel,
                onSelect = { onFilterChange(LookupFilterField.LIFT, it) },
            )
        }
    }
}

@Composable
private fun FilterSummaryBar(
    selectorState: LookupSelectorState,
    expanded: Boolean,
    onToggle: () -> Unit,
    enabled: Boolean,
    onFilterChange: (LookupFilterField, String) -> Unit,
) {
    val filters = selectorState.filters
    val options = selectorState.options
    val summary = buildFilterSummary(filters)

    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
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
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    Text(
                        text = "Cohort",
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.primary,
                    )
                    Text(
                        text = summary,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
                FilterChip(
                    selected = expanded,
                    onClick = onToggle,
                    label = { Text(if (expanded) "Done" else "Refine") },
                )
            }

            if (expanded) {
                Column(
                    modifier = Modifier.padding(top = 14.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                ) {
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
        title = "Filters",
        eyebrow = "Refine cohort",
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
                    modifier = Modifier.height(40.dp),
                    label = {
                        Text(labelFor(option))
                    },
                )
            }
        }
    }
}

@Composable
private fun PercentileLookupCard(
    preview: HistogramLookupPreview,
    crossSexPreview: CrossSexLookupPresentation?,
) {
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
        title = "Your result",
        eyebrow = "Percentile lookup",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(
                text = "${preview.liftLabel} in ${preview.metric} from ${formatCount(preview.histogram.total)} lifters",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Surface(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.72f),
                ) {
                    Column(
                        modifier = Modifier.padding(14.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Text(
                            text = "Median",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        Text(
                            text = "${preview.p50?.let(::formatMetricValue) ?: "n/a"} ${preview.metric.lowercase(Locale.US)}",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                    }
                }
                Surface(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.72f),
                ) {
                    Column(
                        modifier = Modifier.padding(14.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Text(
                            text = "Top 10%",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                        Text(
                            text = "${preview.p90?.let(::formatMetricValue) ?: "n/a"} ${preview.metric.lowercase(Locale.US)}",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                    }
                }
            }
            OutlinedTextField(
                value = liftInput,
                onValueChange = { liftInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Your ${preview.liftLabel.lowercase(Locale.US)} (${preview.metric})") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            )
            when {
                liftInput.isBlank() -> {
                    Text(
                        text = "Enter your best lift to see where you rank.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                parsedLift == null -> {
                    Text(
                        text = "Enter a valid number, e.g. 500 or 182.5.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }

                lookup != null -> {
                    val percentileLabel = formatPercent(lookup.percentile * 100f)
                    val strengthSummary = percentileInterpretation(lookup.percentile)

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
                                text = "You're stronger than $percentileLabel of similar lifters",
                                style = MaterialTheme.typography.headlineMedium,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                            Text(
                                text = "$strengthSummary  •  Rank ${formatCount(lookup.rank)} of ${formatCount(lookup.total)}",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                            )
                        }
                    }
                    NextMilestoneHint(
                        currentPercentile = lookup.percentile,
                        preview = preview,
                    )
                    ShareResultButton(
                        percentileLabel = percentileLabel,
                        strengthSummary = strengthSummary,
                        liftValue = liftInput,
                        preview = preview,
                    )
                }
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Text(
                text = "Compare against your bodyweight",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "See how you rank among lifters near your bodyweight.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            OutlinedTextField(
                value = bodyweightInput,
                onValueChange = { bodyweightInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Your bodyweight (kg)") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            )
            when {
                preview.heatmap == null -> {}

                bodyweightInput.isBlank() -> {}

                parsedBodyweight == null -> {
                    Text(
                        text = "Enter a valid bodyweight, e.g. 90 or 82.5.",
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
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            DistributionCard(
                preview = preview,
                inputValue = parsedLift,
                bodyweightKg = parsedBodyweight,
            )
            crossSexPreview?.let { comparison ->
                CrossSexComparisonCard(
                    preview = comparison,
                    inputValue = parsedLift,
                )
            }
        }
    }
}

@Composable
private fun ShareResultButton(
    percentileLabel: String,
    strengthSummary: String,
    liftValue: String,
    preview: HistogramLookupPreview,
) {
    val context = LocalContext.current
    Button(
        onClick = {
            val text = "I'm in the $percentileLabel ($strengthSummary) for ${preview.cohortLabel} " +
                "with a ${preview.liftLabel.lowercase(Locale.US)} of $liftValue ${preview.metric}. " +
                "Checked on Iron Insights."
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = "text/plain"
                putExtra(Intent.EXTRA_TEXT, text)
            }
            context.startActivity(Intent.createChooser(intent, "Share your result"))
        },
        modifier = Modifier.fillMaxWidth(),
        colors = ButtonDefaults.buttonColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
            contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
        ),
    ) {
        Text("Share this result")
    }
}

@Composable
private fun NextMilestoneHint(
    currentPercentile: Float,
    preview: HistogramLookupPreview,
) {
    val nextThreshold = when {
        currentPercentile < 0.50f -> 0.50f
        currentPercentile < 0.75f -> 0.75f
        currentPercentile < 0.90f -> 0.90f
        currentPercentile < 0.95f -> 0.95f
        else -> null
    }
    if (nextThreshold == null) return

    val nextValue = com.gregorycarnegie.ironinsights.data.repository.valueForPercentile(
        preview.histogram,
        nextThreshold,
    ) ?: return
    val label = formatPercent(nextThreshold * 100f)
    val metric = preview.metric.lowercase(Locale.US)

    Surface(
        shape = RoundedCornerShape(14.dp),
        color = MaterialTheme.colorScheme.tertiaryContainer,
    ) {
        Text(
            modifier = Modifier.padding(14.dp),
            text = "Next goal: ${formatMetricValue(nextValue)} $metric to reach the $label mark.",
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onTertiaryContainer,
        )
    }
}

@Composable
private fun BodyweightConditionedCard(
    preview: HistogramLookupPreview,
    lookup: BodyweightConditionedLookup,
) {
    val strengthSummary = percentileInterpretation(lookup.percentile)

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
                text = "Stronger than ${formatPercent(lookup.percentile * 100f)} of lifters at ${formatMetricValue(lookup.bwWindowLow)}-${formatMetricValue(lookup.bwWindowHigh)} kg",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Text(
                text = "$strengthSummary  •  Rank ~${formatCount(lookup.rank)} of ${formatCount(lookup.totalNearby)} nearby lifters",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onTertiaryContainer,
            )
        }
    }
}

@Composable
private fun DistributionCard(
    preview: HistogramLookupPreview,
    inputValue: Float?,
    bodyweightKg: Float?,
) {
    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Text(
            text = "Distribution view",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
            fontWeight = FontWeight.SemiBold,
        )
        Text(
            text = "Histogram and bodyweight heatmap for the exact cohort behind your lookup.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        HistogramChart(
            histogram = preview.histogram,
            metric = preview.metric,
            inputValue = inputValue,
        )
        preview.heatmap?.let { heatmap ->
            HeatmapChart(
                heatmap = heatmap,
                metric = preview.metric,
                inputValue = inputValue,
                bodyweightKg = bodyweightKg,
            )
        }
    }
}

@Composable
private fun HistogramChart(
    histogram: com.gregorycarnegie.ironinsights.data.model.HistogramBin,
    metric: String,
    inputValue: Float?,
) {
    val bins = rebinHistogramForDisplay(histogram)
    val maxCount = bins.maxOfOrNull { it.count }?.toFloat()?.coerceAtLeast(1f) ?: 1f
    val axisColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.75f)
    val barColor = MaterialTheme.colorScheme.primary
    val markerColor = MaterialTheme.colorScheme.tertiary

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.74f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = "Histogram",
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(180.dp),
            ) {
                val left = 30f
                val top = 12f
                val right = 8f
                val bottom = 24f
                val plotWidth = (size.width - left - right).coerceAtLeast(1f)
                val plotHeight = (size.height - top - bottom).coerceAtLeast(1f)
                val span = (histogram.max - histogram.min).coerceAtLeast(0.001f)

                drawLine(
                    color = axisColor,
                    start = Offset(left, top + plotHeight),
                    end = Offset(left + plotWidth, top + plotHeight),
                    strokeWidth = 2f,
                )
                drawLine(
                    color = axisColor,
                    start = Offset(left, top),
                    end = Offset(left, top + plotHeight),
                    strokeWidth = 2f,
                )

                repeat(3) { index ->
                    val y = top + plotHeight * (index / 2f)
                    drawLine(
                        color = axisColor,
                        start = Offset(left, y),
                        end = Offset(left + plotWidth, y),
                        strokeWidth = 1f,
                    )
                }

                bins.forEach { bin ->
                    if (bin.count == 0L) return@forEach
                    val x0 = left + ((bin.start - histogram.min) / span) * plotWidth
                    val x1 = left + ((bin.end - histogram.min) / span) * plotWidth
                    val barHeight = (bin.count.toFloat() / maxCount) * plotHeight
                    drawRect(
                        color = barColor,
                        topLeft = Offset(x0, top + plotHeight - barHeight),
                        size = androidx.compose.ui.geometry.Size(
                            width = (x1 - x0).coerceAtLeast(1f),
                            height = barHeight,
                        ),
                    )
                }

                inputValue?.let { value ->
                    val markerX = left + ((value - histogram.min) / span).coerceIn(0f, 1f) * plotWidth
                    drawLine(
                        color = markerColor,
                        start = Offset(markerX, top),
                        end = Offset(markerX, top + plotHeight),
                        strokeWidth = 4f,
                    )
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = formatMetricValue(histogram.min),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = metricOptionLabel(metric),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = formatMetricValue(histogram.max),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun HeatmapChart(
    heatmap: com.gregorycarnegie.ironinsights.data.model.HeatmapBin,
    metric: String,
    inputValue: Float?,
    bodyweightKg: Float?,
) {
    val cells = rebinHeatmapForDisplay(heatmap)
    val maxCell = cells.maxOfOrNull { it.count }?.toFloat()?.coerceAtLeast(1f) ?: 1f
    val axisColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.75f)
    val heatColor = MaterialTheme.colorScheme.secondary
    val markerColor = MaterialTheme.colorScheme.tertiary

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.74f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = "Bodyweight heatmap",
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(220.dp),
            ) {
                val left = 34f
                val top = 12f
                val right = 10f
                val bottom = 28f
                val plotWidth = (size.width - left - right).coerceAtLeast(1f)
                val plotHeight = (size.height - top - bottom).coerceAtLeast(1f)
                val xSpan = (heatmap.maxX - heatmap.minX).coerceAtLeast(0.001f)
                val ySpan = (heatmap.maxY - heatmap.minY).coerceAtLeast(0.001f)

                drawLine(
                    color = axisColor,
                    start = Offset(left, top + plotHeight),
                    end = Offset(left + plotWidth, top + plotHeight),
                    strokeWidth = 2f,
                )
                drawLine(
                    color = axisColor,
                    start = Offset(left, top),
                    end = Offset(left, top + plotHeight),
                    strokeWidth = 2f,
                )

                cells.forEach { cell ->
                    if (cell.count == 0L) return@forEach
                    val x0 = left + ((cell.x0 - heatmap.minX) / xSpan) * plotWidth
                    val x1 = left + ((cell.x1 - heatmap.minX) / xSpan) * plotWidth
                    val y0 = top + plotHeight - ((cell.y1 - heatmap.minY) / ySpan) * plotHeight
                    val y1 = top + plotHeight - ((cell.y0 - heatmap.minY) / ySpan) * plotHeight
                    drawRect(
                        color = heatColor.copy(
                            alpha = (0.12f + 0.88f * (cell.count.toFloat() / maxCell)).coerceIn(0.12f, 1f),
                        ),
                        topLeft = Offset(x0, y0),
                        size = androidx.compose.ui.geometry.Size(
                            width = (x1 - x0).coerceAtLeast(1f),
                            height = (y1 - y0).coerceAtLeast(1f),
                        ),
                    )
                }

                if (inputValue != null && bodyweightKg != null) {
                    val markerX = left + ((inputValue - heatmap.minX) / xSpan).coerceIn(0f, 1f) * plotWidth
                    val markerY = top + plotHeight - ((bodyweightKg - heatmap.minY) / ySpan).coerceIn(0f, 1f) * plotHeight
                    drawLine(
                        color = markerColor,
                        start = Offset(markerX, top),
                        end = Offset(markerX, top + plotHeight),
                        strokeWidth = 3f,
                    )
                    drawLine(
                        color = markerColor,
                        start = Offset(left, markerY),
                        end = Offset(left + plotWidth, markerY),
                        strokeWidth = 3f,
                    )
                    drawCircle(
                        color = markerColor,
                        radius = 6f,
                        center = Offset(markerX, markerY),
                    )
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = "${formatMetricValue(heatmap.minY)} kg",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "Bodyweight vs ${metricOptionLabel(metric)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "${formatMetricValue(heatmap.maxY)} kg",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

private data class DisplayHistogramBin(
    val start: Float,
    val end: Float,
    val count: Long,
)

private data class DisplayHeatCell(
    val x0: Float,
    val x1: Float,
    val y0: Float,
    val y1: Float,
    val count: Long,
)

private fun rebinHistogramForDisplay(
    histogram: com.gregorycarnegie.ironinsights.data.model.HistogramBin,
    maxBins: Int = 48,
): List<DisplayHistogramBin> {
    val counts = histogram.counts
    if (counts.isEmpty()) {
        return emptyList()
    }
    val groupSize = kotlin.math.ceil(counts.size / maxBins.toFloat()).toInt().coerceAtLeast(1)
    return counts.chunked(groupSize).mapIndexed { index, chunk ->
        val start = histogram.min + index * groupSize * histogram.baseBin
        val end = (start + chunk.size * histogram.baseBin).coerceAtMost(histogram.max)
        DisplayHistogramBin(
            start = start,
            end = end,
            count = chunk.sum(),
        )
    }
}

private fun rebinHeatmapForDisplay(
    heatmap: com.gregorycarnegie.ironinsights.data.model.HeatmapBin,
    maxWidth: Int = 32,
    maxHeight: Int = 24,
): List<DisplayHeatCell> {
    if (heatmap.width <= 0 || heatmap.height <= 0 || heatmap.grid.isEmpty()) {
        return emptyList()
    }
    val xGroup = kotlin.math.ceil(heatmap.width / maxWidth.toFloat()).toInt().coerceAtLeast(1)
    val yGroup = kotlin.math.ceil(heatmap.height / maxHeight.toFloat()).toInt().coerceAtLeast(1)
    val cells = mutableListOf<DisplayHeatCell>()
    var groupedY = 0
    while (groupedY < heatmap.height) {
        var groupedX = 0
        while (groupedX < heatmap.width) {
            var total = 0L
            val xEndIndex = minOf(groupedX + xGroup, heatmap.width)
            val yEndIndex = minOf(groupedY + yGroup, heatmap.height)
            for (y in groupedY until yEndIndex) {
                for (x in groupedX until xEndIndex) {
                    total += heatmap.grid[y * heatmap.width + x]
                }
            }
            cells += DisplayHeatCell(
                x0 = heatmap.minX + groupedX * heatmap.baseX,
                x1 = (heatmap.minX + xEndIndex * heatmap.baseX).coerceAtMost(heatmap.maxX),
                y0 = heatmap.minY + groupedY * heatmap.baseY,
                y1 = (heatmap.minY + yEndIndex * heatmap.baseY).coerceAtMost(heatmap.maxY),
                count = total,
            )
            groupedX += xGroup
        }
        groupedY += yGroup
    }
    return cells
}

@Composable
private fun CrossSexComparisonCard(
    preview: CrossSexLookupPresentation,
    inputValue: Float?,
) {
    val maleLookup = inputValue?.let { percentileForValue(preview.male.histogram, it) }
    val femaleLookup = inputValue?.let { percentileForValue(preview.female.histogram, it) }
    val femaleEquivalent = maleLookup?.let {
        com.gregorycarnegie.ironinsights.data.repository.valueForPercentile(
            preview.female.histogram,
            it.percentile,
        )
    }
    val maleEquivalent = femaleLookup?.let {
        com.gregorycarnegie.ironinsights.data.repository.valueForPercentile(
            preview.male.histogram,
            it.percentile,
        )
    }

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.78f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Men vs women",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Same lift, same equipment, same tested status, compared side by side across male and female cohorts.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (inputValue == null) {
                Text(
                    text = "Enter a lift above to compare how the same score lands in both cohorts.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            } else if (maleLookup != null && femaleLookup != null) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    CrossSexStatCard(
                        modifier = Modifier.weight(1f),
                        title = "Men",
                        percentile = formatPercent(maleLookup.percentile * 100f),
                        cohortSize = formatCount(maleLookup.total),
                        note = preview.male.note,
                    )
                    CrossSexStatCard(
                        modifier = Modifier.weight(1f),
                        title = "Women",
                        percentile = formatPercent(femaleLookup.percentile * 100f),
                        cohortSize = formatCount(femaleLookup.total),
                        note = preview.female.note,
                    )
                }
                femaleEquivalent?.let { value ->
                    Text(
                        text = "Women's equivalent at the same men's percentile: ${formatMetricValue(value)} ${preview.metric.lowercase(Locale.US)}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
                maleEquivalent?.let { value ->
                    Text(
                        text = "Men's equivalent at the same women's percentile: ${formatMetricValue(value)} ${preview.metric.lowercase(Locale.US)}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
        }
    }
}

@Composable
private fun CrossSexStatCard(
    modifier: Modifier = Modifier,
    title: String,
    percentile: String,
    cohortSize: String,
    note: String?,
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.7f),
    ) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = percentile,
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                fontWeight = FontWeight.Bold,
            )
            Text(
                text = "$cohortSize lifters",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            note?.let {
                Text(
                    text = it,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                )
            }
        }
    }
}

@Composable
private fun TrendsCard(trends: TrendSeriesPresentation) {
    val latest = trends.points.lastOrNull() ?: return
    val first = trends.points.firstOrNull()

    SectionCard(
        title = "Trends at a glance",
        eyebrow = "Yearly cohort",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            if (first != null && latest.total > first.total) {
                Text(
                    text = "This cohort grew from ${formatCount(first.total.toLong())} to ${formatCount(latest.total.toLong())} lifters since ${first.year}.",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
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
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.96f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
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
                color = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
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
private fun JourneyLinks(
    onRouteChange: (AppRoute) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Button(
            onClick = { onRouteChange(AppRoute.TRENDS) },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("See how this cohort has changed over time")
        }
        Button(
            onClick = { onRouteChange(AppRoute.COMPARE) },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Compare to similar cohorts")
        }
    }
}

@Composable
private fun DevInfoSection(
    expanded: Boolean,
    onToggle: () -> Unit,
    environment: EnvironmentConfig,
    uiState: HomeUiState,
    endpoints: List<DatasetEndpoint>,
    milestones: List<String>,
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
                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(
                        text = "Data sources & developer info",
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    uiState.latest?.let {
                        Text(
                            text = "Dataset ${it.version}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                FilterChip(
                    selected = expanded,
                    onClick = onToggle,
                    label = { Text(if (expanded) "Hide" else "Show") },
                )
            }

            if (expanded) {
                Column(
                    modifier = Modifier.padding(top = 14.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                ) {
                    Surface(
                        shape = RoundedCornerShape(14.dp),
                        color = MaterialTheme.colorScheme.background.copy(alpha = 0.72f),
                    ) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(14.dp),
                            verticalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            Text(
                                text = "Base URL",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.primary,
                            )
                            Text(
                                text = environment.siteBaseUrl,
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                        }
                    }

                    uiState.latest?.let { latest ->
                        Text(
                            text = "Version: ${latest.version}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        latest.revision?.let {
                            Text(
                                text = "Revision: $it",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }

                    uiState.rootShardCount?.let { shardCount ->
                        Text(
                            text = "Root index shards: $shardCount",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                    uiState.shardPreview?.let { preview ->
                        Text(
                            text = "Current shard: ${preview.shardKey} (${preview.sliceCount} slices)",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                    uiState.trendsPreview?.let { trends ->
                        Text(
                            text = "Trends: ${trends.seriesCount} series, bucket=${trends.bucket}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                    uiState.loadSummary?.let { summary ->
                        LoadSourcesCard(summary = summary)
                    }

                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

                    Text(
                        text = "Dataset endpoints",
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    endpoints.forEach { endpoint ->
                        EndpointRow(endpoint = endpoint)
                    }

                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

                    Text(
                        text = "Milestones",
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    milestones.forEach { milestone ->
                        MilestoneRow(text = milestone)
                    }

                    Button(
                        onClick = onRefresh,
                        enabled = !uiState.isLoading,
                    ) {
                        Text(if (uiState.isLoading) "Loading..." else "Refresh data")
                    }
                }
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
            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
        ),
        shape = RoundedCornerShape(24.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.18f)
                    .height(3.dp)
                    .background(
                        brush = Brush.horizontalGradient(
                            colors = listOf(
                                MaterialTheme.colorScheme.primary,
                                MaterialTheme.colorScheme.secondary,
                            ),
                        ),
                        shape = RoundedCornerShape(99.dp),
                    ),
            )
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = eyebrow,
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.primary,
                )
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface,
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

private fun buildFilterSummary(filters: LookupFilters): String {
    return listOf(
        sexOptionLabel(filters.sex),
        filters.equip,
        if (filters.wc == "All") "All bodyweights" else "${filters.wc} kg",
        if (filters.age == "All Ages") "All ages" else ageOptionLabel(filters.age),
        liftOptionLabel(filters.lift),
        metricOptionLabel(filters.metric),
    ).joinToString(" · ")
}

private fun percentileInterpretation(percentile: Float): String {
    return when {
        percentile >= 0.99f -> "Elite"
        percentile >= 0.95f -> "Exceptional"
        percentile >= 0.90f -> "Advanced"
        percentile >= 0.75f -> "Above average"
        percentile >= 0.50f -> "Intermediate"
        percentile >= 0.25f -> "Developing"
        else -> "Beginner"
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
