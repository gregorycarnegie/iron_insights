package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.relation.ExerciseWithSets
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import java.util.Locale

@Composable
fun WorkoutLogScreen(
    viewModel: WorkoutLogViewModel,
    weightUnit: WeightUnit = WeightUnit.KG,
    modifier: Modifier = Modifier,
) {
    val state = viewModel.uiState

    Scaffold(
        modifier = modifier,
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            MaterialTheme.colorScheme.background,
                            MaterialTheme.colorScheme.surface,
                        ),
                    ),
                ),
        ) {
            when {
                state.isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center),
                        color = MaterialTheme.colorScheme.primary,
                    )
                }

                state.activeSession == null -> {
                    StartWorkoutContent(
                        onStartWorkout = { viewModel.startNewSession() },
                        modifier = Modifier.align(Alignment.Center),
                    )
                }

                else -> {
                    Column(modifier = Modifier.fillMaxSize()) {
                        RestTimerBar(
                            remainingSeconds = state.restTimerSeconds,
                            isRunning = state.restTimerRunning,
                            onStop = { viewModel.stopRestTimer() },
                        )
                        ActiveWorkoutContent(
                            state = state,
                            weightUnit = weightUnit,
                            onAddExercise = { viewModel.showExercisePicker() },
                            onFinishWorkout = { viewModel.finishSession() },
                            onLogSet = { epId, w, r, rpe ->
                                viewModel.logSet(epId, w, r, rpe)
                            },
                            onDeleteSet = { set -> viewModel.deleteSet(set) },
                            onRemoveExercise = { exercise ->
                                viewModel.removeExercise(exercise)
                            },
                            onStartRestTimer = { seconds -> viewModel.startRestTimer(seconds) },
                            modifier = Modifier.weight(1f),
                        )
                    }
                }
            }

            if (state.showExercisePicker) {
                val filteredExercises = if (state.exerciseSearchQuery.isBlank()) {
                    state.exerciseLibrary
                } else {
                    state.exerciseLibrary.filter {
                        it.name.contains(state.exerciseSearchQuery, ignoreCase = true)
                    }
                }

                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.4f)),
                ) {
                    ExercisePickerSheet(
                        exercises = filteredExercises,
                        searchQuery = state.exerciseSearchQuery,
                        onSearchQueryChange = { viewModel.setExerciseSearchQuery(it) },
                        onExerciseSelected = { viewModel.addExercise(it) },
                        onDismiss = { viewModel.hideExercisePicker() },
                        modifier = Modifier.align(Alignment.BottomCenter),
                    )
                }
            }
        }
    }
}

@Composable
private fun StartWorkoutContent(
    onStartWorkout: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = "Ready to train?",
            style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onBackground,
            fontWeight = FontWeight.Bold,
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Start a workout to begin logging sets.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(24.dp))
        Button(
            onClick = onStartWorkout,
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
            ),
            shape = RoundedCornerShape(12.dp),
            modifier = Modifier.fillMaxWidth(0.6f),
        ) {
            Text(
                text = "Start Workout",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(vertical = 4.dp),
            )
        }
    }
}

@Composable
private fun ActiveWorkoutContent(
    state: WorkoutLogUiState,
    weightUnit: WeightUnit,
    onAddExercise: () -> Unit,
    onFinishWorkout: () -> Unit,
    onLogSet: (Long, Float, Int, Float?) -> Unit,
    onDeleteSet: (com.gregorycarnegie.ironinsights.data.db.entity.SetEntry) -> Unit,
    onRemoveExercise: (com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed) -> Unit,
    onStartRestTimer: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            SessionHeader(
                session = state.activeSession!!,
                sessionVolume = state.sessionVolume,
                weightUnit = weightUnit,
            )
        }

        items(state.exercises, key = { it.exercise.id }) { exerciseWithSets ->
            ExerciseCard(
                exerciseWithSets = exerciseWithSets,
                weightUnit = weightUnit,
                onLogSet = { w, r, rpe ->
                    onLogSet(exerciseWithSets.exercise.id, w, r, rpe)
                },
                onDeleteSet = onDeleteSet,
                onRemoveExercise = { onRemoveExercise(exerciseWithSets.exercise) },
                onStartRestTimer = onStartRestTimer,
            )
        }

        item {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                OutlinedButton(
                    onClick = onAddExercise,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                ) {
                    Text("+ Add Exercise")
                }
                Button(
                    onClick = onFinishWorkout,
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                    ),
                    shape = RoundedCornerShape(12.dp),
                ) {
                    Text("Finish Workout")
                }
            }
        }

        item { Spacer(modifier = Modifier.height(32.dp)) }
    }
}

@Composable
private fun SessionHeader(
    session: com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession,
    sessionVolume: com.gregorycarnegie.ironinsights.domain.training.VolumeCalculator.SessionVolume?,
    weightUnit: WeightUnit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(16.dp),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = session.title ?: "Workout",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.Bold,
            )
            Spacer(modifier = Modifier.height(4.dp))
            val elapsed = (System.currentTimeMillis() - session.startedAtEpochMs) / 1000
            val elapsedMin = elapsed / 60
            Text(
                text = "${elapsedMin}m elapsed",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (sessionVolume != null && sessionVolume.totalSets > 0) {
                Spacer(modifier = Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text(
                        text = "${sessionVolume.totalSets} sets",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                    Text(
                        text = "${sessionVolume.totalReps} reps",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                    Text(
                        text = "${formatWeightInput(sessionVolume.totalVolumeLoad)} ${weightUnit.label} volume",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    }
}

@Composable
private fun ExerciseCard(
    exerciseWithSets: ExerciseWithSets,
    weightUnit: WeightUnit,
    onLogSet: (Float, Int, Float?) -> Unit,
    onDeleteSet: (com.gregorycarnegie.ironinsights.data.db.entity.SetEntry) -> Unit,
    onRemoveExercise: () -> Unit,
    onStartRestTimer: (Int) -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Exercise #${exerciseWithSets.exercise.orderIndex + 1}",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    fontWeight = FontWeight.SemiBold,
                )
                TextButton(onClick = onRemoveExercise) {
                    Text(
                        text = "Remove",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }

            // Set header row
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 4.dp),
            ) {
                Text(
                    "#", style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.weight(0.8f),
                )
                Text(
                    "Weight", style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.weight(1.5f),
                )
                Text(
                    "Reps", style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.weight(0.8f),
                )
                Text(
                    "RPE", style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.weight(0.8f),
                )
                Text(
                    "e1RM", style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.weight(1.2f),
                )
                Spacer(modifier = Modifier.width(24.dp))
            }

            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

            // Existing sets
            exerciseWithSets.sets.forEachIndexed { index, set ->
                SetEntryRow(
                    setNumber = index + 1,
                    set = set,
                    weightUnit = weightUnit,
                    onDelete = { onDeleteSet(set) },
                )
            }

            HorizontalDivider(
                modifier = Modifier.padding(vertical = 4.dp),
                color = MaterialTheme.colorScheme.outlineVariant,
            )

            // Add set input row
            AddSetRow(
                lastSet = exerciseWithSets.sets.lastOrNull(),
                weightUnit = weightUnit,
                onLogSet = onLogSet,
                onStartRestTimer = onStartRestTimer,
            )
        }
    }
}

@Composable
private fun AddSetRow(
    lastSet: com.gregorycarnegie.ironinsights.data.db.entity.SetEntry?,
    weightUnit: WeightUnit,
    onLogSet: (Float, Int, Float?) -> Unit,
    onStartRestTimer: (Int) -> Unit,
) {
    var weightText by remember(lastSet) {
        mutableStateOf(
            lastSet?.let {
                formatWeightInput(
                    com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay(
                        it.weightKg,
                        weightUnit,
                    ),
                )
            } ?: "",
        )
    }
    var repsText by remember(lastSet) {
        mutableStateOf(lastSet?.reps?.toString() ?: "")
    }
    var rpeText by remember { mutableStateOf("") }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = weightText,
            onValueChange = { weightText = it },
            label = { Text(weightUnit.label) },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            singleLine = true,
            modifier = Modifier.weight(1f),
        )
        OutlinedTextField(
            value = repsText,
            onValueChange = { repsText = it },
            label = { Text("Reps") },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            singleLine = true,
            modifier = Modifier.weight(1f),
        )
        OutlinedTextField(
            value = rpeText,
            onValueChange = { rpeText = it },
            label = { Text("RPE") },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            singleLine = true,
            modifier = Modifier.weight(0.8f),
        )
        FilledTonalButton(
            onClick = {
                val weight = weightText.toFloatOrNull() ?: return@FilledTonalButton
                val reps = repsText.toIntOrNull() ?: return@FilledTonalButton
                val rpe = rpeText.toFloatOrNull()
                val weightKg = com.gregorycarnegie.ironinsights.domain.calculators.displayToKg(
                    weight,
                    weightUnit,
                )
                onLogSet(weightKg, reps, rpe)
                onStartRestTimer(90)
            },
            shape = RoundedCornerShape(8.dp),
        ) {
            Text("Log")
        }
    }
}
