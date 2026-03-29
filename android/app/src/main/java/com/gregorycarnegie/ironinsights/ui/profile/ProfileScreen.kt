package com.gregorycarnegie.ironinsights.ui.profile

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.ui.home.SectionCard
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

@Composable
fun ProfileScreen(
    uiState: ProfileUiState,
    onNavigateBack: () -> Unit,
    onStartEditing: () -> Unit,
    onCancelEditing: () -> Unit,
    onSaveProfile: () -> Unit,
    onUpdateSex: (String) -> Unit,
    onUpdateBodyweight: (String) -> Unit,
    onUpdateHeight: (String) -> Unit,
    onUpdateAge: (String) -> Unit,
    onUpdateEquipment: (String) -> Unit,
    onUpdateTested: (String) -> Unit,
    onUpdateSquat: (String) -> Unit,
    onUpdateBench: (String) -> Unit,
    onUpdateDeadlift: (String) -> Unit,
) {
    val prefs = uiState.preferences
    val weightLabel = prefs.weightUnit.label

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
                        text = "Profile",
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f),
                    )
                    if (!uiState.isEditing) {
                        Button(onClick = onStartEditing) {
                            Text("Edit")
                        }
                    }
                }
            }

            val squatKg = prefs.squatKg
            val benchKg = prefs.benchKg
            val deadliftKg = prefs.deadliftKg
            if (squatKg != null || benchKg != null || deadliftKg != null) {
                item {
                    SectionCard(title = "Big three", eyebrow = "Lift overview") {
                        LiftRadarChart(
                            squatKg = squatKg ?: 0f,
                            benchKg = benchKg ?: 0f,
                            deadliftKg = deadliftKg ?: 0f,
                            weightLabel = weightLabel,
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(1f),
                        )
                    }
                }
            }

            if (uiState.isEditing) {
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
                                            selected = uiState.sexInput == value,
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
                                            selected = uiState.equipmentInput == value,
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
                                            selected = uiState.testedInput == value,
                                            onClick = { onUpdateTested(value) },
                                            label = { Text(if (value == "Yes") "Tested" else "All") },
                                        )
                                    }
                                }
                            }
                        }
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

                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End),
                    ) {
                        OutlinedButton(onClick = onCancelEditing) {
                            Text("Cancel")
                        }
                        Button(onClick = onSaveProfile) {
                            Text("Save")
                        }
                    }
                }
            } else {
                item {
                    SectionCard(title = "Details", eyebrow = "Body metrics") {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            ProfileDetailRow("Sex", if (prefs.sex == "M") "Male" else if (prefs.sex == "F") "Female" else "--")
                            ProfileDetailRow("Bodyweight", prefs.bodyweightKg?.let {
                                "%.1f $weightLabel".format(kgToDisplay(it, prefs.weightUnit))
                            } ?: "--")
                            ProfileDetailRow("Height", prefs.heightCm?.let { "%.0f cm".format(it) } ?: "--")
                            ProfileDetailRow("Age", prefs.age?.toString() ?: "--")
                            ProfileDetailRow("Equipment", prefs.equipment.ifEmpty { "--" })
                            ProfileDetailRow("Tested", when (prefs.tested) {
                                "Yes" -> "Tested"
                                "All" -> "All"
                                else -> prefs.tested.ifEmpty { "--" }
                            })
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ProfileDetailRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun LiftRadarChart(
    squatKg: Float,
    benchKg: Float,
    deadliftKg: Float,
    weightLabel: String,
    modifier: Modifier = Modifier,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val outlineColor = MaterialTheme.colorScheme.outlineVariant
    val onSurfaceArgb = MaterialTheme.colorScheme.onSurface.toArgb()

    val labels = listOf("Squat", "Bench", "Deadlift")
    val values = listOf(squatKg, benchKg, deadliftKg)
    val maxValue = values.max().coerceAtLeast(1f)

    val textPaint = remember(onSurfaceArgb) {
        android.graphics.Paint().apply {
            color = onSurfaceArgb
            textSize = 38f
            textAlign = android.graphics.Paint.Align.CENTER
            isAntiAlias = true
        }
    }
    val smallPaint = remember(onSurfaceArgb) {
        android.graphics.Paint().apply {
            color = onSurfaceArgb
            textSize = 32f
            textAlign = android.graphics.Paint.Align.CENTER
            isAntiAlias = true
        }
    }

    Canvas(modifier = modifier.padding(32.dp)) {
        val centerX = size.width / 2f
        val centerY = size.height / 2f
        val radius = minOf(centerX, centerY) * 0.75f
        val axisCount = 3
        // Start from top (-PI/2) so Squat axis points upward
        val angleStep = (2 * PI / axisCount).toFloat()
        val startAngle = (-PI / 2f).toFloat()

        fun angleFor(i: Int) = startAngle + i * angleStep
        fun pointAt(i: Int, r: Float) = Offset(
            centerX + r * cos(angleFor(i)),
            centerY + r * sin(angleFor(i)),
        )

        val gridLevels = 5
        for (level in 1..gridLevels) {
            val r = radius * level / gridLevels
            val path = Path()
            for (i in 0 until axisCount) {
                val pt = pointAt(i, r)
                if (i == 0) path.moveTo(pt.x, pt.y) else path.lineTo(pt.x, pt.y)
            }
            path.close()
            drawPath(path, color = outlineColor.copy(alpha = 0.3f), style = Stroke(width = 1f))
        }

        for (i in 0 until axisCount) {
            val end = pointAt(i, radius)
            drawLine(
                color = outlineColor.copy(alpha = 0.5f),
                start = Offset(centerX, centerY),
                end = end,
                strokeWidth = 1f,
            )
        }

        val dataPath = Path()
        for (i in values.indices) {
            val normalized = (values[i] / maxValue).coerceIn(0f, 1f)
            val pt = pointAt(i, radius * normalized)
            if (i == 0) dataPath.moveTo(pt.x, pt.y) else dataPath.lineTo(pt.x, pt.y)
        }
        dataPath.close()

        drawPath(path = dataPath, color = primaryColor.copy(alpha = 0.2f), style = Fill)
        drawPath(path = dataPath, color = primaryColor, style = Stroke(width = 3f))

        for (i in values.indices) {
            val normalized = (values[i] / maxValue).coerceIn(0f, 1f)
            val pt = pointAt(i, radius * normalized)
            drawCircle(color = primaryColor, radius = 6f, center = pt)
            drawCircle(color = Color.White, radius = 3f, center = pt)
        }

        for (i in labels.indices) {
            val labelOffset = pointAt(i, radius + 40f)
            drawContext.canvas.nativeCanvas.drawText(labels[i], labelOffset.x, labelOffset.y, textPaint)
            if (values[i] > 0f) {
                val displayVal = kgToDisplay(values[i], if (weightLabel == "lb") WeightUnit.LB else WeightUnit.KG)
                drawContext.canvas.nativeCanvas.drawText(
                    "%.0f $weightLabel".format(displayVal),
                    labelOffset.x,
                    labelOffset.y + 40f,
                    smallPaint,
                )
            }
        }
    }
}
