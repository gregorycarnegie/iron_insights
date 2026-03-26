package com.gregorycarnegie.ironinsights.ui.programmes

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.domain.training.ExercisePrescription
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeLiftBaseline
import com.gregorycarnegie.ironinsights.domain.training.ProgrammePrescriptionPlan
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeSessionPrescription
import com.gregorycarnegie.ironinsights.domain.training.ProgrammeWeekPrescription
import com.gregorycarnegie.ironinsights.domain.training.SetPrescription
import kotlin.math.roundToInt

@Composable
fun ProgrammeDetailScreen(
    programmeWithBlocks: ProgrammeWithBlocks?,
    generatedPlan: ProgrammePrescriptionPlan?,
    liftBaselines: List<ProgrammeLiftBaseline>,
    weightUnit: WeightUnit,
    onSetActiveProgramme: () -> Unit,
    onAddBlock: (String, String, Int) -> Unit,
    onAddBlockSequence: (List<ProgrammeBlockDraft>) -> Unit,
    onDeleteBlock: (ProgrammeBlock) -> Unit,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val background = Brush.verticalGradient(
        colors = listOf(
            MaterialTheme.colorScheme.background,
            MaterialTheme.colorScheme.surface,
        ),
    )
    val blocks = programmeWithBlocks?.blocks?.sortedBy { it.orderIndex } ?: emptyList()

    Scaffold(modifier = modifier) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(background),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TextButton(onClick = onNavigateBack) {
                        Text(
                            text = "\u2190 Back",
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }

            item {
                ProgrammeHeader(
                    programmeWithBlocks = programmeWithBlocks,
                    onSetActiveProgramme = onSetActiveProgramme,
                )
            }

            item {
                PlanningModesCard()
            }

            item {
                PresetLibraryCard(
                    onAddPreset = onAddBlockSequence,
                )
            }

            item {
                CustomBlockBuilderCard(
                    onAddBlock = onAddBlock,
                )
            }

            if (blocks.isEmpty()) {
                item {
                    EmptyProgrammeCard()
                }
            } else {
                item {
                    Text(
                        text = "Current sequence",
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onBackground,
                        fontWeight = FontWeight.SemiBold,
                    )
                }

                itemsIndexed(blocks, key = { _, block -> block.id }) { index, block ->
                    val startWeek = blocks.take(index).sumOf { it.weekCount } + 1
                    BlockCard(
                        block = block,
                        startWeek = startWeek,
                        onDelete = { onDeleteBlock(block) },
                    )
                }

                item {
                    LiftBaselineCard(
                        baselines = liftBaselines,
                        weightUnit = weightUnit,
                    )
                }

                generatedPlan?.let { plan ->
                    item {
                        GeneratedPlanIntroCard(plan = plan)
                    }

                    itemsIndexed(
                        items = plan.weeks,
                        key = { _, week -> "${week.weekNumber}-${week.blockType}-${week.weekInBlock}" },
                    ) { _, week ->
                        WeekPrescriptionCard(
                            week = week,
                            weightUnit = weightUnit,
                        )
                    }
                }
            }

            item { Spacer(modifier = Modifier.height(32.dp)) }
        }
    }
}

@Composable
private fun ProgrammeHeader(
    programmeWithBlocks: ProgrammeWithBlocks?,
    onSetActiveProgramme: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = programmeWithBlocks?.programme?.name ?: "Programme",
                style = MaterialTheme.typography.headlineMedium,
                color = MaterialTheme.colorScheme.onBackground,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.weight(1f),
            )
            if (programmeWithBlocks?.programme?.isActive == true) {
                Surface(
                    shape = RoundedCornerShape(999.dp),
                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.16f),
                ) {
                    Text(
                        text = "ACTIVE",
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            } else {
                OutlinedButton(
                    onClick = onSetActiveProgramme,
                    shape = RoundedCornerShape(12.dp),
                ) {
                    Text("Set Active")
                }
            }
        }
        programmeWithBlocks?.programme?.notes?.let { notes ->
            Text(
                text = notes,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Text(
            text = if (programmeWithBlocks?.programme?.isActive == true) {
                "This programme drives the guided workout suggestion on the Log tab."
            } else {
                "Set this as active if you want Log to start the next programmed workout automatically."
            },
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun PlanningModesCard() {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = "How professionals periodize",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Traditional cycles usually move from volume to strength to peak. Block cycles usually move from accumulation to intensification to realization, then taper.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "Daily or weekly undulation still matters inside those blocks. This screen handles the macrocycle layer so you can lay out the broad sequence first.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun PresetLibraryCard(
    onAddPreset: (List<ProgrammeBlockDraft>) -> Unit,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Research-backed presets",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Start from a proven sequence, then customize block lengths to match your calendar and recovery.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            programmePresetTemplates.forEach { preset ->
                PresetCard(
                    preset = preset,
                    onAddPreset = { onAddPreset(preset.blocks) },
                )
            }
        }
    }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun PresetCard(
    preset: ProgrammePresetTemplate,
    onAddPreset: () -> Unit,
) {
    Surface(
        shape = RoundedCornerShape(14.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
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
                        text = preset.title,
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.onSurface,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = preset.summary,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Surface(
                    shape = RoundedCornerShape(999.dp),
                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f),
                ) {
                    Text(
                        text = "${programmePresetTotalWeeks(preset)} weeks",
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            FlowRow(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                preset.blocks.forEach { block ->
                    val blockTemplate = programmeBlockTemplate(block.blockType)
                    Surface(
                        shape = RoundedCornerShape(999.dp),
                        color = blockAccentColor(block.blockType).copy(alpha = 0.16f),
                    ) {
                        Text(
                            text = "${blockTemplate.label} ${block.weekCount}w",
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                            style = MaterialTheme.typography.labelSmall,
                            color = blockAccentColor(block.blockType),
                        )
                    }
                }
            }

            OutlinedButton(
                onClick = onAddPreset,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp),
            ) {
                Text("Add This Sequence")
            }
        }
    }
}

@Composable
private fun CustomBlockBuilderCard(
    onAddBlock: (String, String, Int) -> Unit,
) {
    var selectedBlockType by rememberSaveable { mutableStateOf(programmeBlockTemplates.first().blockType) }
    val selectedTemplate = programmeBlockTemplate(selectedBlockType)
    var customName by rememberSaveable(selectedBlockType) { mutableStateOf(selectedTemplate.defaultName) }
    var weekCount by rememberSaveable(selectedBlockType) { mutableStateOf(selectedTemplate.defaultWeeks) }

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Custom block builder",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Choose the block type first, then set the duration inside its recommended range.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            LazyBlockTypeRow(
                selectedBlockType = selectedBlockType,
                onSelect = {
                    selectedBlockType = it
                },
            )

            OutlinedTextField(
                value = customName,
                onValueChange = { customName = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Block name") },
                singleLine = true,
            )

            Surface(
                shape = RoundedCornerShape(14.dp),
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = selectedTemplate.label,
                        style = MaterialTheme.typography.titleSmall,
                        color = blockAccentColor(selectedTemplate.blockType),
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = selectedTemplate.goal,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Text(
                        text = selectedTemplate.loadingFocus,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Text(
                        text = selectedTemplate.rationale,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Duration",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "Recommended ${selectedTemplate.weekRange.first}-${selectedTemplate.weekRange.last} weeks",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                OutlinedButton(
                    onClick = {
                        weekCount = (weekCount - 1).coerceAtLeast(selectedTemplate.weekRange.first)
                    },
                    enabled = weekCount > selectedTemplate.weekRange.first,
                    shape = RoundedCornerShape(10.dp),
                ) {
                    Text("-")
                }
                Surface(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(12.dp),
                    color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.72f),
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 12.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = "$weekCount weeks",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
                OutlinedButton(
                    onClick = {
                        weekCount = (weekCount + 1).coerceAtMost(selectedTemplate.weekRange.last)
                    },
                    enabled = weekCount < selectedTemplate.weekRange.last,
                    shape = RoundedCornerShape(10.dp),
                ) {
                    Text("+")
                }
            }

            Button(
                onClick = {
                    onAddBlock(
                        customName.trim().ifBlank { selectedTemplate.defaultName },
                        selectedTemplate.blockType,
                        weekCount,
                    )
                },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.primary,
                ),
                shape = RoundedCornerShape(12.dp),
            ) {
                Text("Add Block")
            }
        }
    }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun LazyBlockTypeRow(
    selectedBlockType: String,
    onSelect: (String) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        maxItemsInEachRow = 3,
    ) {
        programmeBlockTemplates.forEach { template ->
            FilterChip(
                selected = template.blockType == selectedBlockType,
                onClick = { onSelect(template.blockType) },
                label = { Text(template.label) },
            )
        }
    }
}

@Composable
private fun EmptyProgrammeCard() {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = "No blocks yet",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Seed the programme with a preset sequence or build it block by block. You are no longer limited to hypertrophy or 4-week phases.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun BlockCard(
    block: ProgrammeBlock,
    startWeek: Int,
    onDelete: () -> Unit,
) {
    val template = programmeBlockTemplate(block.blockType)
    val endWeek = startWeek + block.weekCount - 1

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Text(
                        text = block.name,
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Surface(
                            shape = RoundedCornerShape(999.dp),
                            color = blockAccentColor(block.blockType).copy(alpha = 0.16f),
                        ) {
                            Text(
                                text = template.label,
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                                style = MaterialTheme.typography.labelSmall,
                                color = blockAccentColor(block.blockType),
                            )
                        }
                        Surface(
                            shape = RoundedCornerShape(999.dp),
                            color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f),
                        ) {
                            Text(
                                text = "Weeks $startWeek-$endWeek",
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.primary,
                            )
                        }
                    }
                }
                TextButton(onClick = onDelete) {
                    Text(
                        text = "Remove",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }

            Spacer(modifier = Modifier.height(10.dp))
            Text(
                text = "${block.weekCount} weeks  •  ${template.goal}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = template.loadingFocus,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = template.rationale,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun LiftBaselineCard(
    baselines: List<ProgrammeLiftBaseline>,
    weightUnit: WeightUnit,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Auto-selected baselines",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = "Exact working weights come from your latest estimated maxes. If a lift has no history yet, the programme falls back to %1RM until you log one.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            FlowRow(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                baselines.forEach { baseline ->
                    Surface(
                        shape = RoundedCornerShape(14.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
                    ) {
                        Column(
                            modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            Text(
                                text = baseline.label,
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                            Text(
                                text = baseline.e1rmKg?.let { displayWeight(it, weightUnit) } ?: "No baseline yet",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSurface,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun GeneratedPlanIntroCard(
    plan: ProgrammePrescriptionPlan,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = "Ready-made weekly plan",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = plan.summary,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (plan.missingBaselines.isNotEmpty()) {
                Text(
                    text = "Still waiting on: ${plan.missingBaselines.joinToString(", ")}. Those lifts will show percentages until you log them.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.tertiary,
                )
            }
        }
    }
}

@Composable
private fun WeekPrescriptionCard(
    week: ProgrammeWeekPrescription,
    weightUnit: WeightUnit,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(18.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
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
                        text = "Week ${week.weekNumber}",
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onSurface,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = "${week.blockName} • ${programmeBlockTemplate(week.blockType).label} • Block week ${week.weekInBlock}/${week.totalWeeksInBlock}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Surface(
                    shape = RoundedCornerShape(999.dp),
                    color = blockAccentColor(week.blockType).copy(alpha = 0.16f),
                ) {
                    Text(
                        text = programmeBlockTemplate(week.blockType).label,
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                        style = MaterialTheme.typography.labelMedium,
                        color = blockAccentColor(week.blockType),
                    )
                }
            }

            Text(
                text = week.focus,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            week.sessions.forEach { session ->
                SessionPrescriptionCard(
                    session = session,
                    weightUnit = weightUnit,
                )
            }
        }
    }
}

@Composable
private fun SessionPrescriptionCard(
    session: ProgrammeSessionPrescription,
    weightUnit: WeightUnit,
) {
    Surface(
        shape = RoundedCornerShape(14.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = session.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    fontWeight = FontWeight.SemiBold,
                )
                Surface(
                    shape = RoundedCornerShape(999.dp),
                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f),
                ) {
                    Text(
                        text = session.dayLabel,
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            Text(
                text = session.coachingNote,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = session.warmupNote,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.primary,
            )

            session.exercises.forEach { exercise ->
                ExercisePrescriptionBlock(
                    exercise = exercise,
                    weightUnit = weightUnit,
                )
            }
        }
    }
}

@Composable
private fun ExercisePrescriptionBlock(
    exercise: ExercisePrescription,
    weightUnit: WeightUnit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            text = exercise.exerciseName,
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.onSurface,
            fontWeight = FontWeight.SemiBold,
        )
        exercise.lines.forEach { line ->
            Text(
                text = formatPrescriptionLine(
                    line = line,
                    weightUnit = weightUnit,
                ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        exercise.note?.let { note ->
            Text(
                text = note,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.tertiary,
            )
        }
    }
}

private fun formatPrescriptionLine(
    line: SetPrescription,
    weightUnit: WeightUnit,
): String {
    val loadText = line.targetWeightKg?.let { displayWeight(it, weightUnit) }
        ?: line.intensityPercent?.let { "${formatPercent(it)} e1RM" }
        ?: "Self-limit"
    val rpeText = line.targetRpe?.let { " @ RPE ${formatWeightInput(it)}" } ?: ""
    return "${line.label}: ${line.sets} x ${line.reps} @ $loadText$rpeText • rest ${formatRest(line.restSeconds)}"
}

private fun displayWeight(
    weightKg: Float,
    weightUnit: WeightUnit,
): String {
    return "${formatWeightInput(kgToDisplay(weightKg, weightUnit))} ${weightUnit.label}"
}

private fun formatPercent(value: Float): String {
    return "${(value * 100f).roundToInt()}%"
}

private fun formatRest(seconds: Int): String {
    val minutes = seconds / 60
    val remainder = seconds % 60
    return if (remainder == 0) {
        "${minutes}:00"
    } else {
        "$minutes:${remainder.toString().padStart(2, '0')}"
    }
}

@Composable
private fun blockAccentColor(blockType: String): Color {
    return when (blockType) {
        "hypertrophy", "accumulation" -> MaterialTheme.colorScheme.secondary
        "strength", "intensification" -> MaterialTheme.colorScheme.primary
        "realization", "peak" -> MaterialTheme.colorScheme.error
        "taper", "deload" -> MaterialTheme.colorScheme.tertiary
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }
}
