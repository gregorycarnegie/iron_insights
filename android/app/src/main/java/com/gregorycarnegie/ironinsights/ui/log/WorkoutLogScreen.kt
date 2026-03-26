package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
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
import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition
import com.gregorycarnegie.ironinsights.data.db.relation.SessionWithExercises
import com.gregorycarnegie.ironinsights.data.db.relation.ExerciseWithSets
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.domain.training.ExercisePrescription
import com.gregorycarnegie.ironinsights.domain.training.ProgrammedWorkoutMetadata
import com.gregorycarnegie.ironinsights.domain.training.ProgrammedWorkoutRecommendation
import com.gregorycarnegie.ironinsights.domain.training.SetPrescription
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

@Composable
fun WorkoutLogScreen(
    viewModel: WorkoutLogViewModel,
    weightUnit: WeightUnit = WeightUnit.KG,
    modifier: Modifier = Modifier,
) {
    val state = viewModel.uiState
    val exerciseNameById = remember(state.exerciseLibrary) {
        state.exerciseLibrary.associateBy(ExerciseDefinition::id)
    }

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
                        onStartProgrammedWorkout = { viewModel.startProgrammedWorkout() },
                        recommendedProgrammedWorkout = state.recommendedProgrammedWorkout,
                        recentSessions = state.recentSessions.filter { it.session.finishedAtEpochMs != null },
                        weightUnit = weightUnit,
                        exerciseNameFor = { exerciseId ->
                            exerciseNameById[exerciseId]?.name ?: "Exercise #$exerciseId"
                        },
                        modifier = Modifier.fillMaxSize(),
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
                            exerciseNameFor = { exerciseId ->
                                exerciseNameById[exerciseId]?.name ?: "Exercise #$exerciseId"
                            },
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
    onStartProgrammedWorkout: () -> Unit,
    recommendedProgrammedWorkout: ProgrammedWorkoutRecommendation?,
    recentSessions: List<SessionWithExercises>,
    weightUnit: WeightUnit,
    exerciseNameFor: (Long) -> String,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier,
        contentPadding = PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp),
    ) {
        item {
            if (recommendedProgrammedWorkout != null) {
                ProgrammedStartCard(
                    recommendation = recommendedProgrammedWorkout,
                    onStartProgrammedWorkout = onStartProgrammedWorkout,
                    onStartManualWorkout = onStartWorkout,
                )
            } else {
                Surface(
                    shape = RoundedCornerShape(20.dp),
                    color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.78f),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text(
                            text = "Ready to train?",
                            style = MaterialTheme.typography.headlineMedium,
                            color = MaterialTheme.colorScheme.onBackground,
                            fontWeight = FontWeight.Bold,
                        )
                        Text(
                            text = "Start a workout to log sets, then review recent sessions below.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Button(
                            onClick = onStartWorkout,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                            ),
                            shape = RoundedCornerShape(12.dp),
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Text(
                                text = "Start Workout",
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.padding(vertical = 4.dp),
                            )
                        }
                    }
                }
            }
        }

        if (recentSessions.isNotEmpty()) {
            item {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(
                        text = "Previous workouts",
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onBackground,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = "Recent completed sessions stay visible here even when no workout is active.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            items(recentSessions, key = { it.session.id }) { sessionWithExercises ->
                WorkoutHistoryCard(
                    sessionWithExercises = sessionWithExercises,
                    weightUnit = weightUnit,
                    exerciseNameFor = exerciseNameFor,
                )
            }
        }
    }
}

@Composable
private fun ProgrammedStartCard(
    recommendation: ProgrammedWorkoutRecommendation,
    onStartProgrammedWorkout: () -> Unit,
    onStartManualWorkout: () -> Unit,
) {
    val exerciseSummary = recommendation.session.exercises
        .take(3)
        .joinToString(" • ") { it.exerciseName }

    Surface(
        shape = RoundedCornerShape(20.dp),
        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.86f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Programmed workout ready",
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                fontWeight = FontWeight.Bold,
            )
            Text(
                text = "${recommendation.metadata.programmeName} • Week ${recommendation.metadata.weekNumber} • ${recommendation.metadata.dayLabel}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = recommendation.session.title,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = exerciseSummary,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Button(
                onClick = onStartProgrammedWorkout,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.primary,
                ),
            ) {
                Text("Start Programmed Workout")
            }
            OutlinedButton(
                onClick = onStartManualWorkout,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp),
            ) {
                Text("Start Manual Workout")
            }
        }
    }
}

@Composable
private fun WorkoutHistoryCard(
    sessionWithExercises: SessionWithExercises,
    weightUnit: WeightUnit,
    exerciseNameFor: (Long) -> String,
) {
    val session = sessionWithExercises.session
    val stats = historyStats(sessionWithExercises)
    val exerciseSummary = sessionWithExercises.exercises
        .map { exerciseNameFor(it.exercise.exerciseId) }
        .distinct()
        .take(4)
        .joinToString(", ")

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = session.title ?: "Workout",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "${formatSessionTimestamp(session.startedAtEpochMs)}  •  ${formatSessionDuration(session.startedAtEpochMs, session.finishedAtEpochMs)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                HistoryStatChip(
                    modifier = Modifier.weight(1f),
                    label = "Exercises",
                    value = stats.exerciseCount.toString(),
                )
                HistoryStatChip(
                    modifier = Modifier.weight(1f),
                    label = "Sets",
                    value = stats.setCount.toString(),
                )
                HistoryStatChip(
                    modifier = Modifier.weight(1f),
                    label = "Volume",
                    value = "${formatWeightInput(kgToDisplay(stats.volumeKg, weightUnit))} ${weightUnit.label}",
                )
            }
            if (exerciseSummary.isNotBlank()) {
                Text(
                    text = exerciseSummary,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun HistoryStatChip(
    modifier: Modifier = Modifier,
    label: String,
    value: String,
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

private data class WorkoutHistoryStats(
    val exerciseCount: Int,
    val setCount: Int,
    val volumeKg: Float,
)

private fun historyStats(sessionWithExercises: SessionWithExercises): WorkoutHistoryStats {
    val setCount = sessionWithExercises.exercises.sumOf { it.sets.size }
    val volumeKg = sessionWithExercises.exercises
        .flatMap { it.sets }
        .sumOf { set -> (set.weightKg * set.reps).toDouble() }
        .toFloat()
    return WorkoutHistoryStats(
        exerciseCount = sessionWithExercises.exercises.size,
        setCount = setCount,
        volumeKg = volumeKg,
    )
}

private fun formatSessionTimestamp(epochMs: Long): String {
    val formatter = SimpleDateFormat("EEE d MMM • HH:mm", Locale.US)
    return formatter.format(Date(epochMs))
}

private fun formatSessionDuration(
    startedAtEpochMs: Long,
    finishedAtEpochMs: Long?,
): String {
    val end = finishedAtEpochMs ?: return "In progress"
    val totalMinutes = ((end - startedAtEpochMs) / 60_000L).coerceAtLeast(0L)
    val hours = totalMinutes / 60
    val minutes = totalMinutes % 60
    return when {
        hours > 0 -> "${hours}h ${minutes}m"
        else -> "${minutes}m"
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
    exerciseNameFor: (Long) -> String,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            SessionHeader(
                session = state.activeSession!!,
                programmedSessionMetadata = state.programmedSessionMetadata,
                sessionVolume = state.sessionVolume,
                weightUnit = weightUnit,
            )
        }

        items(state.exercises, key = { it.exercise.id }) { exerciseWithSets ->
            ExerciseCard(
                exerciseWithSets = exerciseWithSets,
                exerciseName = exerciseNameFor(exerciseWithSets.exercise.exerciseId),
                weightUnit = weightUnit,
                prescription = state.programmedExercisePrescriptions[exerciseWithSets.exercise.id],
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
    programmedSessionMetadata: ProgrammedWorkoutMetadata?,
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
            programmedSessionMetadata?.let { metadata ->
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "${metadata.programmeName} • Week ${metadata.weekNumber} • ${metadata.dayLabel}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
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
    exerciseName: String,
    weightUnit: WeightUnit,
    prescription: ExercisePrescription?,
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
                    text = exerciseName,
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

            prescription?.let { guided ->
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "Prescription",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.SemiBold,
                )
                guided.lines.forEach { line ->
                    Text(
                        text = formatGuidedLine(line, weightUnit),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                guided.note?.let { note ->
                    Text(
                        text = note,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.tertiary,
                    )
                }
                Spacer(modifier = Modifier.height(6.dp))
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
                prescription = prescription,
                completedSetCount = exerciseWithSets.sets.size,
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
    prescription: ExercisePrescription?,
    completedSetCount: Int,
    weightUnit: WeightUnit,
    onLogSet: (Float, Int, Float?) -> Unit,
    onStartRestTimer: (Int) -> Unit,
) {
    val nextPrescriptionLine = nextPrescriptionLine(
        prescription = prescription,
        completedSetCount = completedSetCount,
    )

    var weightText by remember(lastSet, nextPrescriptionLine, weightUnit) {
        mutableStateOf(
            nextPrescriptionLine?.targetWeightKg?.let {
                formatWeightInput(
                    com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay(
                        it,
                        weightUnit,
                    ),
                )
            } ?: lastSet?.let {
                formatWeightInput(
                    com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay(
                        it.weightKg,
                        weightUnit,
                    ),
                )
            } ?: "",
        )
    }
    var repsText by remember(lastSet, nextPrescriptionLine) {
        mutableStateOf(
            nextPrescriptionLine?.reps?.toString()
                ?: lastSet?.reps?.toString()
                ?: "",
        )
    }
    var rpeText by remember(nextPrescriptionLine) {
        mutableStateOf(nextPrescriptionLine?.targetRpe?.let { formatWeightInput(it) } ?: "")
    }

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
                onStartRestTimer(nextPrescriptionLine?.restSeconds ?: 90)
            },
            shape = RoundedCornerShape(8.dp),
        ) {
            Text("Log")
        }
    }
}

private fun formatGuidedLine(
    line: SetPrescription,
    weightUnit: WeightUnit,
): String {
    val load = line.targetWeightKg?.let {
        "${formatWeightInput(kgToDisplay(it, weightUnit))} ${weightUnit.label}"
    } ?: line.intensityPercent?.let {
        "${(it * 100f).toInt()}% e1RM"
    } ?: "Self-limit"
    val rpe = line.targetRpe?.let { " @ RPE ${formatWeightInput(it)}" } ?: ""
    return "${line.label}: ${line.sets} x ${line.reps} @ $load$rpe • rest ${formatRestDuration(line.restSeconds)}"
}

private fun nextPrescriptionLine(
    prescription: ExercisePrescription?,
    completedSetCount: Int,
): SetPrescription? {
    if (prescription == null || prescription.lines.isEmpty()) return null

    var consumedSets = 0
    prescription.lines.forEach { line ->
        val boundary = consumedSets + line.sets
        if (completedSetCount < boundary) {
            return line
        }
        consumedSets = boundary
    }
    return prescription.lines.last()
}

private fun formatRestDuration(seconds: Int): String {
    val minutes = seconds / 60
    val remainder = seconds % 60
    return "$minutes:${remainder.toString().padStart(2, '0')}"
}
