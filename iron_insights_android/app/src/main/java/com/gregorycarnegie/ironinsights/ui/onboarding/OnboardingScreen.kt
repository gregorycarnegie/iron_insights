package com.gregorycarnegie.ironinsights.ui.onboarding

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.ui.home.SectionCard

@Composable
fun OnboardingScreen(
    uiState: OnboardingUiState,
    onUpdateSex: (String) -> Unit,
    onUpdateBodyweight: (String) -> Unit,
    onUpdateHeight: (String) -> Unit,
    onUpdateAge: (String) -> Unit,
    onUpdateEquipment: (String) -> Unit,
    onUpdateTested: (String) -> Unit,
    onUpdateSquat: (String) -> Unit,
    onUpdateBench: (String) -> Unit,
    onUpdateDeadlift: (String) -> Unit,
    onNext: () -> Unit,
    onBack: () -> Unit,
    onFinish: () -> Unit,
    onSkip: () -> Unit,
) {
    val background = Brush.verticalGradient(
        colors = listOf(
            MaterialTheme.colorScheme.background,
            MaterialTheme.colorScheme.surface,
            MaterialTheme.colorScheme.primaryContainer,
        ),
    )

    val stepIndex = if (uiState.step == OnboardingStep.BODY_METRICS) 0 else 1
    val progress = (stepIndex + 1) / 2f

    Scaffold(containerColor = MaterialTheme.colorScheme.background) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(background)
                .padding(innerPadding),
        ) {
            LinearProgressIndicator(
                progress = { progress },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 12.dp),
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
            )

            AnimatedContent(
                targetState = uiState.step,
                transitionSpec = {
                    if (targetState.ordinal > initialState.ordinal) {
                        slideInHorizontally { it } togetherWith slideOutHorizontally { -it }
                    } else {
                        slideInHorizontally { -it } togetherWith slideOutHorizontally { it }
                    }
                },
                modifier = Modifier.weight(1f),
                label = "onboarding_step",
            ) { step ->
                when (step) {
                    OnboardingStep.BODY_METRICS -> BodyMetricsStep(
                        uiState = uiState,
                        onUpdateSex = onUpdateSex,
                        onUpdateBodyweight = onUpdateBodyweight,
                        onUpdateHeight = onUpdateHeight,
                        onUpdateAge = onUpdateAge,
                        onUpdateEquipment = onUpdateEquipment,
                        onUpdateTested = onUpdateTested,
                    )
                    OnboardingStep.LIFT_NUMBERS -> LiftNumbersStep(
                        uiState = uiState,
                        onUpdateSquat = onUpdateSquat,
                        onUpdateBench = onUpdateBench,
                        onUpdateDeadlift = onUpdateDeadlift,
                    )
                }
            }

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 16.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (uiState.step == OnboardingStep.BODY_METRICS) {
                    TextButton(onClick = onSkip) {
                        Text("Skip")
                    }
                    Button(onClick = onNext) {
                        Text("Next")
                    }
                } else {
                    OutlinedButton(onClick = onBack) {
                        Text("Back")
                    }
                    Button(
                        onClick = onFinish,
                        enabled = !uiState.isSaving,
                    ) {
                        Text(if (uiState.isSaving) "Saving..." else "Get started")
                    }
                }
            }
        }
    }
}

@Composable
private fun BodyMetricsStep(
    uiState: OnboardingUiState,
    onUpdateSex: (String) -> Unit,
    onUpdateBodyweight: (String) -> Unit,
    onUpdateHeight: (String) -> Unit,
    onUpdateAge: (String) -> Unit,
    onUpdateEquipment: (String) -> Unit,
    onUpdateTested: (String) -> Unit,
) {
    val weightLabel = uiState.weightUnit.label

    LazyColumn(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(22.dp),
    ) {
        item {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    text = "Welcome to Iron Insights",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "Tell us about yourself so we can personalise your experience.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        item {
            SectionCard(title = "About you", eyebrow = "Body metrics") {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = "Sex",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(listOf("M", "F"), key = { it }) { value ->
                                FilterChip(
                                    selected = uiState.sex == value,
                                    onClick = { onUpdateSex(value) },
                                    label = { Text(if (value == "M") "Male" else "Female") },
                                )
                            }
                        }
                    }

                    OutlinedTextField(
                        value = uiState.bodyweightInput,
                        onValueChange = onUpdateBodyweight,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Bodyweight ($weightLabel)") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )

                    OutlinedTextField(
                        value = uiState.heightInput,
                        onValueChange = onUpdateHeight,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Height (cm)") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )

                    OutlinedTextField(
                        value = uiState.ageInput,
                        onValueChange = onUpdateAge,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Age") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    )
                }
            }
        }

        item {
            SectionCard(title = "Training style", eyebrow = "Competition") {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = "Equipment",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(listOf("Raw", "Wraps", "Single-ply", "Multi-ply"), key = { it }) { value ->
                                FilterChip(
                                    selected = uiState.equipment == value,
                                    onClick = { onUpdateEquipment(value) },
                                    label = { Text(value) },
                                )
                            }
                        }
                    }

                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = "Drug tested",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(listOf("All", "Yes"), key = { it }) { value ->
                                FilterChip(
                                    selected = uiState.tested == value,
                                    onClick = { onUpdateTested(value) },
                                    label = { Text(if (value == "Yes") "Tested" else "All") },
                                )
                            }
                        }
                    }
                }
            }
        }

        item { Spacer(Modifier.height(8.dp)) }
    }
}

@Composable
private fun LiftNumbersStep(
    uiState: OnboardingUiState,
    onUpdateSquat: (String) -> Unit,
    onUpdateBench: (String) -> Unit,
    onUpdateDeadlift: (String) -> Unit,
) {
    val weightLabel = uiState.weightUnit.label

    LazyColumn(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(22.dp),
    ) {
        item {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    text = "Your big three",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "Enter your current or estimated 1RM for each lift. You can always update these later.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        item {
            SectionCard(title = "Lifts", eyebrow = "Estimated 1RM") {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    OutlinedTextField(
                        value = uiState.squatInput,
                        onValueChange = onUpdateSquat,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Squat ($weightLabel)") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )

                    OutlinedTextField(
                        value = uiState.benchInput,
                        onValueChange = onUpdateBench,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Bench press ($weightLabel)") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )

                    OutlinedTextField(
                        value = uiState.deadliftInput,
                        onValueChange = onUpdateDeadlift,
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Deadlift ($weightLabel)") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )
                }
            }
        }

        item { Spacer(Modifier.height(8.dp)) }
    }
}
