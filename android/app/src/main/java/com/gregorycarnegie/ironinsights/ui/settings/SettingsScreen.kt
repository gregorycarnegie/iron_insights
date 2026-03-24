package com.gregorycarnegie.ironinsights.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
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
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.domain.calculators.displayToKg
import com.gregorycarnegie.ironinsights.ui.home.SectionCard

@Composable
fun SettingsScreen(
    uiState: SettingsUiState,
    onUpdateWeightUnit: (WeightUnit) -> Unit,
    onUpdateBarWeight: (Float) -> Unit,
    onUpdateRoundingIncrement: (Float) -> Unit,
    onUpdateHealthConnectEnabled: (Boolean) -> Unit,
    onNavigateBack: () -> Unit,
) {
    val prefs = uiState.preferences

    var barWeightInput by rememberSaveable(prefs.barWeightKg, prefs.weightUnit) {
        mutableStateOf(formatWeightInput(kgToDisplay(prefs.barWeightKg, prefs.weightUnit)))
    }
    var roundingInput by rememberSaveable(prefs.roundingIncrement, prefs.weightUnit) {
        mutableStateOf(formatWeightInput(kgToDisplay(prefs.roundingIncrement, prefs.weightUnit)))
    }

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
            verticalArrangement = Arrangement.spacedBy(22.dp),
        ) {
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    IconButton(onClick = onNavigateBack) {
                        Text(
                            text = "\u2190",
                            style = MaterialTheme.typography.titleLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                    }
                    Text(
                        text = "Settings",
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }

            item {
                SectionCard(
                    title = "Units & defaults",
                    eyebrow = "Preferences",
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text(
                                text = "Weight unit",
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                            LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                items(WeightUnit.entries, key = { it.name }) { unit ->
                                    FilterChip(
                                        selected = prefs.weightUnit == unit,
                                        onClick = { onUpdateWeightUnit(unit) },
                                        label = { Text(unit.label) },
                                    )
                                }
                            }
                        }

                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

                        OutlinedTextField(
                            value = barWeightInput,
                            onValueChange = { input ->
                                barWeightInput = input
                                input.toFloatOrNull()?.takeIf { it >= 0f }?.let { parsed ->
                                    onUpdateBarWeight(displayToKg(parsed, prefs.weightUnit))
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                            label = { Text("Default bar weight (${prefs.weightUnit.label})") },
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            supportingText = {
                                Text("Used as the default bar weight in the plate calculator.")
                            },
                        )

                        OutlinedTextField(
                            value = roundingInput,
                            onValueChange = { input ->
                                roundingInput = input
                                input.toFloatOrNull()?.takeIf { it > 0f }?.let { parsed ->
                                    onUpdateRoundingIncrement(displayToKg(parsed, prefs.weightUnit))
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                            label = { Text("Rounding increment (${prefs.weightUnit.label})") },
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            supportingText = {
                                Text("Weights will be rounded to the nearest multiple of this value.")
                            },
                        )
                    }
                }
            }

            item {
                SectionCard(
                    title = "Integrations",
                    eyebrow = "Connected services",
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(
                            modifier = Modifier.weight(1f),
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            Text(
                                text = "Health Connect",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                            Text(
                                text = "Sync workout data with Health Connect. Coming soon.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        Switch(
                            checked = prefs.healthConnectEnabled,
                            onCheckedChange = onUpdateHealthConnectEnabled,
                            enabled = false,
                        )
                    }
                }
            }
        }
    }
}
