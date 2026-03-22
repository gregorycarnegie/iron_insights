package com.gregorycarnegie.ironinsights.ui.calculators

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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.domain.calculators.OneRepMaxCalculator
import com.gregorycarnegie.ironinsights.domain.calculators.PlateCalculation
import com.gregorycarnegie.ironinsights.domain.calculators.PlateCalculator
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.displayToKg
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.ui.home.SectionCard
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.navigation.AppRouteTabs
import java.util.Locale
import kotlin.math.abs

@Composable
fun CalculatorsScreen(
    selectedRoute: AppRoute,
    onRouteChange: (AppRoute) -> Unit,
) {
    var unit by rememberSaveable { mutableStateOf(WeightUnit.KG) }

    var workingLoadKg by rememberSaveable { mutableStateOf(100f) }
    var workingLoadInput by rememberSaveable { mutableStateOf("100") }
    var reps by rememberSaveable { mutableStateOf(5) }
    var repsInput by rememberSaveable { mutableStateOf("5") }

    var targetWeightKg by rememberSaveable { mutableStateOf(100f) }
    var targetWeightInput by rememberSaveable { mutableStateOf("100") }
    var barWeightKg by rememberSaveable { mutableStateOf(20f) }
    var barWeightInput by rememberSaveable { mutableStateOf("20") }

    fun switchUnit(nextUnit: WeightUnit) {
        if (nextUnit == unit) {
            return
        }
        unit = nextUnit
        workingLoadInput = formatWeightInput(kgToDisplay(workingLoadKg, nextUnit))
        targetWeightInput = formatWeightInput(kgToDisplay(targetWeightKg, nextUnit))
        barWeightInput = formatWeightInput(kgToDisplay(barWeightKg, nextUnit))
    }

    val oneRmEstimateKg = OneRepMaxCalculator.blended1rm(workingLoadKg, reps)
    val oneRmRange = OneRepMaxCalculator.formulaRange(workingLoadKg, reps)
    val setIntensity = OneRepMaxCalculator.setIntensityPercent(workingLoadKg, reps)
    val plateCalculation = PlateCalculator.calculate(targetWeightKg, barWeightKg)

    val background = Brush.verticalGradient(
        colors = listOf(
            MaterialTheme.colorScheme.background,
            MaterialTheme.colorScheme.surface,
            MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = 0.76f),
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
                    title = "Training tools",
                    eyebrow = "Local calculators",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text(
                            text = "These screens do not hit the website data bundle. They mirror the web app's local calculator behavior inside Android.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "Both calculators keep kg as the internal base even when you switch to pounds for input and display.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        UnitToggleRow(
                            selectedUnit = unit,
                            onUnitChange = ::switchUnit,
                        )
                    }
                }
            }

            item {
                OneRepMaxCard(
                    unit = unit,
                    workingLoadInput = workingLoadInput,
                    onWorkingLoadInputChange = { input ->
                        workingLoadInput = input
                        input.toFloatOrNull()?.takeIf { it > 0f }?.let { parsed ->
                            workingLoadKg = displayToKg(parsed, unit)
                        }
                    },
                    repsInput = repsInput,
                    onRepsInputChange = { input ->
                        repsInput = input
                        input.toIntOrNull()?.takeIf { it in OneRepMaxCalculator.MIN_REPS..OneRepMaxCalculator.MAX_REPS }?.let { parsed ->
                            reps = parsed
                        }
                    },
                    reps = reps,
                    onRepPreset = { preset ->
                        reps = preset
                        repsInput = preset.toString()
                    },
                    estimateKg = oneRmEstimateKg,
                    rangeLowKg = oneRmRange.lowerKg,
                    rangeHighKg = oneRmRange.upperKg,
                    intensityPercent = setIntensity,
                )
            }

            item {
                PlateCalculatorCard(
                    unit = unit,
                    targetWeightInput = targetWeightInput,
                    onTargetWeightInputChange = { input ->
                        targetWeightInput = input
                        input.toFloatOrNull()?.takeIf { it >= 0f }?.let { parsed ->
                            targetWeightKg = displayToKg(parsed, unit)
                        }
                    },
                    barWeightInput = barWeightInput,
                    onBarWeightInputChange = { input ->
                        barWeightInput = input
                        input.toFloatOrNull()?.takeIf { it >= 0f }?.let { parsed ->
                            barWeightKg = displayToKg(parsed, unit)
                        }
                    },
                    targetWeightKg = targetWeightKg,
                    barWeightKg = barWeightKg,
                    calculation = plateCalculation,
                )
            }
        }
    }
}

@Composable
private fun UnitToggleRow(
    selectedUnit: WeightUnit,
    onUnitChange: (WeightUnit) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            text = "Units",
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(WeightUnit.entries, key = { it.name }) { unit ->
                FilterChip(
                    selected = selectedUnit == unit,
                    onClick = { onUnitChange(unit) },
                    label = {
                        Text(unit.label)
                    },
                )
            }
        }
    }
}

@Composable
private fun OneRepMaxCard(
    unit: WeightUnit,
    workingLoadInput: String,
    onWorkingLoadInputChange: (String) -> Unit,
    repsInput: String,
    onRepsInputChange: (String) -> Unit,
    reps: Int,
    onRepPreset: (Int) -> Unit,
    estimateKg: Float,
    rangeLowKg: Float,
    rangeHighKg: Float,
    intensityPercent: Float,
) {
    val badge = OneRepMaxCalculator.estimateQualityBadge(reps)
    val guidance = OneRepMaxCalculator.estimateGuidance(reps)

    SectionCard(
        title = "1RM calculator",
        eyebrow = "Blended estimate",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
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
                        text = badge,
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = displayWeight(estimateKg, unit),
                        style = MaterialTheme.typography.headlineMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = "Estimated 1RM",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = "Common-formula range ${displayWeight(rangeLowKg, unit)}-${displayWeight(rangeHighKg, unit)}.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                    Text(
                        text = guidance,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer.copy(alpha = 0.88f),
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedTextField(
                    value = workingLoadInput,
                    onValueChange = onWorkingLoadInputChange,
                    modifier = Modifier.weight(1f),
                    label = { Text("Lifted weight (${unit.label})") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                )
                OutlinedTextField(
                    value = repsInput,
                    onValueChange = onRepsInputChange,
                    modifier = Modifier.width(132.dp),
                    label = { Text("Reps") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                )
            }

            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "Quick reps",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(OneRepMaxCalculator.quickRepPresets, key = { it }) { preset ->
                        FilterChip(
                            selected = reps == preset,
                            onClick = { onRepPreset(preset) },
                            label = {
                                Text("$preset reps")
                            },
                        )
                    }
                }
            }

            LazyRow(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                item {
                    CalculatorStatCard(
                        title = "Set intensity",
                        value = formatPercent(intensityPercent),
                        subtitle = "How hard the entered set is relative to the estimated max.",
                    )
                }
                item {
                    CalculatorStatCard(
                        title = "80% work sets",
                        value = displayWeight(estimateKg * 0.8f, unit),
                        subtitle = "Useful baseline for steady work.",
                    )
                }
                item {
                    CalculatorStatCard(
                        title = "90% heavy work",
                        value = displayWeight(estimateKg * 0.9f, unit),
                        subtitle = "Heavy single or double territory.",
                    )
                }
            }

            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

            Text(
                text = "Rep max targets",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OneRepMaxCalculator.repMaxTargets.forEach { target ->
                    CalculatorTableRow(
                        title = "${target.reps} reps",
                        note = target.label,
                        value = displayWeight(
                            OneRepMaxCalculator.workingWeightForReps(estimateKg, target.reps),
                            unit,
                        ),
                    )
                }
            }

            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

            Text(
                text = "Training percentages",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OneRepMaxCalculator.trainingPercentages.forEach { percentage ->
                    CalculatorTableRow(
                        title = "${percentage.percent}%",
                        note = percentage.label,
                        value = displayWeight(
                            estimateKg * (percentage.percent.toFloat() / 100f),
                            unit,
                        ),
                    )
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
                        text = "Formula details",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Text(
                        text = "Under 8 reps the estimate follows Brzycki. Above 10 reps it follows Epley. Reps 8-10 blend the two so the output stays smooth.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun PlateCalculatorCard(
    unit: WeightUnit,
    targetWeightInput: String,
    onTargetWeightInputChange: (String) -> Unit,
    barWeightInput: String,
    onBarWeightInputChange: (String) -> Unit,
    targetWeightKg: Float,
    barWeightKg: Float,
    calculation: PlateCalculation,
) {
    val warningText = plateWarningText(
        targetWeightKg = targetWeightKg,
        barWeightKg = barWeightKg,
        remainderKg = calculation.remainderKg,
        unit = unit,
    )

    SectionCard(
        title = "Plate calculator",
        eyebrow = "Load the bar",
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(
                text = "Competition plate set only: 25, 20, 15, 10, 5, 2.5, and 1.25kg. Pounds mode only changes display and input.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedTextField(
                    value = targetWeightInput,
                    onValueChange = onTargetWeightInputChange,
                    modifier = Modifier.weight(1f),
                    label = { Text("Target (${unit.label})") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                )
                OutlinedTextField(
                    value = barWeightInput,
                    onValueChange = onBarWeightInputChange,
                    modifier = Modifier.weight(1f),
                    label = { Text("Bar (${unit.label})") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                )
            }

            PlateStripPreview(
                plates = calculation.plates,
                unit = unit,
                targetWeightKg = targetWeightKg,
                barWeightKg = barWeightKg,
            )

            LazyRow(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                item {
                    CalculatorStatCard(
                        title = "Actual loaded",
                        value = displayWeight(calculation.actualKg, unit),
                        subtitle = "Greedy result with the standard kg plate set.",
                    )
                }
                item {
                    CalculatorStatCard(
                        title = "Remainder",
                        value = signedDisplayWeight(calculation.remainderKg, unit),
                        subtitle = if (abs(calculation.remainderKg) < 0.001f) "Exact target hit." else "Shortfall after the greedy plate pass.",
                    )
                }
            }

            if (warningText != null) {
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.errorContainer,
                ) {
                    Text(
                        modifier = Modifier.padding(16.dp),
                        text = warningText,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onErrorContainer,
                    )
                }
            }

            Text(
                text = "Per side",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
            )

            if (calculation.plates.isEmpty()) {
                Surface(
                    shape = RoundedCornerShape(18.dp),
                    color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
                ) {
                    Text(
                        modifier = Modifier.padding(16.dp),
                        text = if (targetWeightKg <= barWeightKg) "No plates needed. This is effectively bar only." else "No standard competition plate combination fits the remaining load.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    calculation.plates.forEach { load ->
                        PlateBreakdownRow(
                            name = load.plate.name,
                            colorHex = load.plate.colorHex,
                            eachWeightKg = load.plate.weightKg,
                            countPerSide = load.countPerSide,
                            unit = unit,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun PlateStripPreview(
    plates: List<com.gregorycarnegie.ironinsights.domain.calculators.PlateLoad>,
    unit: WeightUnit,
    targetWeightKg: Float,
    barWeightKg: Float,
) {
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
                text = "Side view",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (plates.isEmpty()) {
                Text(
                    text = if (targetWeightKg <= barWeightKg) "Bar only." else "No plate stack available for the entered target.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            } else {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Row(
                        modifier = Modifier.weight(1f),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        plates.reversed().forEach { load ->
                            repeat(load.countPerSide) {
                                PlateCapsule(colorHex = load.plate.colorHex)
                            }
                        }
                    }
                    Box(
                        modifier = Modifier
                            .width(56.dp)
                            .height(10.dp)
                            .background(
                                color = MaterialTheme.colorScheme.onSurface,
                                shape = RoundedCornerShape(99.dp),
                            ),
                    )
                    Row(
                        modifier = Modifier.weight(1f),
                        horizontalArrangement = Arrangement.Start,
                    ) {
                        plates.forEach { load ->
                            repeat(load.countPerSide) {
                                PlateCapsule(colorHex = load.plate.colorHex)
                            }
                        }
                    }
                }
                Text(
                    text = "Colors mirror the competition plate stack per side.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "Target ${displayWeight(targetWeightKg, unit)} with a ${displayWeight(barWeightKg, unit)} bar.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun PlateCapsule(colorHex: String) {
    Surface(
        modifier = Modifier
            .padding(horizontal = 1.dp)
            .width(16.dp)
            .height(64.dp),
        shape = RoundedCornerShape(8.dp),
        color = colorFromHex(colorHex),
    ) {}
}

@Composable
private fun CalculatorStatCard(
    title: String,
    value: String,
    subtitle: String,
) {
    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.84f),
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
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.84f),
            )
        }
    }
}

@Composable
private fun CalculatorTableRow(
    title: String,
    note: String,
    value: String,
) {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = note,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                text = value,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

@Composable
private fun PlateBreakdownRow(
    name: String,
    colorHex: String,
    eachWeightKg: Float,
    countPerSide: Int,
    unit: WeightUnit,
) {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .width(12.dp)
                    .height(44.dp)
                    .background(color = colorFromHex(colorHex), shape = RoundedCornerShape(8.dp)),
            )
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "${displayWeight(eachWeightKg, unit)} each",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                text = "x$countPerSide / side",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

private fun plateWarningText(
    targetWeightKg: Float,
    barWeightKg: Float,
    remainderKg: Float,
    unit: WeightUnit,
): String? {
    return when {
        targetWeightKg < barWeightKg -> {
            "Target is less than bar weight (${displayWeight(barWeightKg, unit)})."
        }

        remainderKg > 0.001f -> {
            "Cannot hit ${displayWeight(targetWeightKg, unit)} exactly with standard competition plates. Short by ${displayWeight(remainderKg, unit)}."
        }

        else -> null
    }
}

private fun displayWeight(
    valueKg: Float,
    unit: WeightUnit,
): String {
    return "${formatWeightInput(kgToDisplay(valueKg, unit))} ${unit.label}"
}

private fun signedDisplayWeight(
    valueKg: Float,
    unit: WeightUnit,
): String {
    val display = kgToDisplay(valueKg, unit)
    val prefix = if (display > 0f) "+" else ""
    return "$prefix${formatWeightInput(display)} ${unit.label}"
}

private fun formatPercent(value: Float): String {
    return String.format(Locale.US, "%.0f%%", value)
}

private fun colorFromHex(hex: String): androidx.compose.ui.graphics.Color {
    return androidx.compose.ui.graphics.Color(android.graphics.Color.parseColor(hex))
}
