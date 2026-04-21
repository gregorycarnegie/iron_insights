package com.gregorycarnegie.ironinsights.ui.equipment

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.BarPreset
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventory
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventoryItem
import com.gregorycarnegie.ironinsights.ui.home.SectionCard

@Composable
fun EquipmentScreen(
    viewModel: EquipmentViewModel,
) {
    val state = viewModel.uiState
    var showNewInventoryDialog by rememberSaveable { mutableStateOf(false) }
    var showNewBarDialog by rememberSaveable { mutableStateOf(false) }
    var showAddPlateDialog by rememberSaveable { mutableStateOf(false) }

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
            verticalArrangement = Arrangement.spacedBy(22.dp),
        ) {
            item {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(
                        text = "Equipment",
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Text(
                        text = "Manage your plate inventories and bar presets.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            // --- Plate Inventories ---
            item {
                SectionCard(title = "Plate inventories", eyebrow = "Your plates") {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        if (state.inventories.isEmpty()) {
                            Text(
                                text = "No inventories yet. Create one to track your available plates.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        } else {
                            state.inventories.forEach { inventory ->
                                InventoryRow(
                                    inventory = inventory,
                                    isActive = inventory.id == state.activeInventoryId,
                                    isSelected = inventory.id == state.selectedInventoryId,
                                    onSelect = { viewModel.selectInventory(inventory.id) },
                                    onSetActive = { viewModel.setActiveInventory(inventory.id) },
                                    onDelete = { viewModel.deleteInventory(inventory) },
                                )
                            }
                        }

                        if (state.activeInventoryId != null) {
                            TextButton(onClick = { viewModel.setActiveInventory(null) }) {
                                Text("Use standard plates")
                            }
                        }

                        OutlinedButton(onClick = { showNewInventoryDialog = true }) {
                            Text("+ New inventory")
                        }
                    }
                }
            }

            // --- Selected inventory items ---
            if (state.selectedInventoryId != null) {
                item {
                    SectionCard(
                        title = state.inventories.find { it.id == state.selectedInventoryId }?.name ?: "Plates",
                        eyebrow = "Inventory contents",
                    ) {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            if (state.selectedInventoryItems.isEmpty()) {
                                Text(
                                    text = "No plates in this inventory. Add some below.",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            } else {
                                state.selectedInventoryItems.forEach { item ->
                                    PlateItemRow(
                                        item = item,
                                        onDelete = { viewModel.deleteItem(item) },
                                    )
                                }
                            }

                            OutlinedButton(onClick = { showAddPlateDialog = true }) {
                                Text("+ Add plate")
                            }
                        }
                    }
                }
            }

            // --- Bar Presets ---
            item {
                SectionCard(title = "Bar presets", eyebrow = "Your bars") {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        if (state.barPresets.isEmpty()) {
                            Text(
                                text = "No bar presets. The default 20kg bar is used.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        } else {
                            state.barPresets.forEach { preset ->
                                BarPresetRow(
                                    preset = preset,
                                    isActive = preset.id == state.activeBarPresetId,
                                    onSetActive = { viewModel.setActiveBarPreset(preset.id) },
                                    onDelete = { viewModel.deleteBarPreset(preset) },
                                )
                            }
                        }

                        if (state.activeBarPresetId != null) {
                            TextButton(onClick = { viewModel.setActiveBarPreset(null) }) {
                                Text("Use default 20kg bar")
                            }
                        }

                        OutlinedButton(onClick = { showNewBarDialog = true }) {
                            Text("+ New bar preset")
                        }
                    }
                }
            }
        }
    }

    if (showNewInventoryDialog) {
        NewInventoryDialog(
            onDismiss = { showNewInventoryDialog = false },
            onCreate = { name ->
                viewModel.createInventory(name)
                showNewInventoryDialog = false
            },
        )
    }

    if (showNewBarDialog) {
        NewBarPresetDialog(
            onDismiss = { showNewBarDialog = false },
            onCreate = { name, weightKg ->
                viewModel.createBarPreset(name, weightKg)
                showNewBarDialog = false
            },
        )
    }

    if (showAddPlateDialog && state.selectedInventoryId != null) {
        AddPlateDialog(
            onDismiss = { showAddPlateDialog = false },
            onAdd = { weightKg, name, colorHex, pairCount ->
                viewModel.addItem(state.selectedInventoryId, weightKg, name, colorHex, pairCount)
                showAddPlateDialog = false
            },
        )
    }
}

@Composable
private fun InventoryRow(
    inventory: PlateInventory,
    isActive: Boolean,
    isSelected: Boolean,
    onSelect: () -> Unit,
    onSetActive: () -> Unit,
    onDelete: () -> Unit,
) {
    Surface(
        shape = RoundedCornerShape(16.dp),
        color = if (isSelected) {
            MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.84f)
        } else {
            MaterialTheme.colorScheme.surface.copy(alpha = 0.9f)
        },
        modifier = Modifier.clickable { onSelect() },
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    text = inventory.name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                if (isActive) {
                    Text(
                        text = "Active",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (!isActive) {
                    TextButton(onClick = onSetActive) { Text("Use") }
                }
                TextButton(onClick = onDelete) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            }
        }
    }
}

@Composable
private fun PlateItemRow(
    item: PlateInventoryItem,
    onDelete: () -> Unit,
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
                    .background(
                        color = colorFromHex(item.colorHex),
                        shape = RoundedCornerShape(8.dp),
                    ),
            )
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    text = item.name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "${item.weightKg}kg — ${item.pairCount} pair${if (item.pairCount != 1) "s" else ""}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            TextButton(onClick = onDelete) {
                Text("Delete", color = MaterialTheme.colorScheme.error)
            }
        }
    }
}

@Composable
private fun BarPresetRow(
    preset: BarPreset,
    isActive: Boolean,
    onSetActive: () -> Unit,
    onDelete: () -> Unit,
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
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    text = preset.name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "${preset.weightKg}kg",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (isActive) {
                    Text(
                        text = "Active",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (!isActive) {
                    TextButton(onClick = onSetActive) { Text("Use") }
                }
                TextButton(onClick = onDelete) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            }
        }
    }
}

@Composable
private fun NewInventoryDialog(
    onDismiss: () -> Unit,
    onCreate: (String) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("New plate inventory") },
        text = {
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("Name") },
                singleLine = true,
            )
        },
        confirmButton = {
            Button(
                onClick = { onCreate(name.trim()) },
                enabled = name.isNotBlank(),
            ) { Text("Create") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}

@Composable
private fun NewBarPresetDialog(
    onDismiss: () -> Unit,
    onCreate: (String, Float) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }
    var weightInput by rememberSaveable { mutableStateOf("20") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("New bar preset") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name") },
                    singleLine = true,
                )
                OutlinedTextField(
                    value = weightInput,
                    onValueChange = { weightInput = it },
                    label = { Text("Weight (kg)") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                )
            }
        },
        confirmButton = {
            val weight = weightInput.toFloatOrNull()
            Button(
                onClick = { weight?.let { onCreate(name.trim(), it) } },
                enabled = name.isNotBlank() && weight != null && weight > 0f,
            ) { Text("Create") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}

@Composable
private fun AddPlateDialog(
    onDismiss: () -> Unit,
    onAdd: (Float, String, String, Int) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }
    var weightInput by rememberSaveable { mutableStateOf("") }
    var pairCountInput by rememberSaveable { mutableStateOf("1") }
    var selectedColor by rememberSaveable { mutableStateOf("#e63946") }

    val colorOptions = listOf(
        "#e63946" to "Red",
        "#457bca" to "Blue",
        "#ffd166" to "Yellow",
        "#5cb85c" to "Green",
        "#444444" to "Black",
        "#ffffff" to "White",
    )

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Add plate") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name") },
                    singleLine = true,
                )
                OutlinedTextField(
                    value = weightInput,
                    onValueChange = { weightInput = it },
                    label = { Text("Weight (kg)") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                )
                OutlinedTextField(
                    value = pairCountInput,
                    onValueChange = { pairCountInput = it },
                    label = { Text("Pairs available") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                )
                Text("Color", style = MaterialTheme.typography.labelLarge)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    colorOptions.forEach { (hex, label) ->
                        FilterChip(
                            selected = selectedColor == hex,
                            onClick = { selectedColor = hex },
                            label = { Text(label) },
                        )
                    }
                }
            }
        },
        confirmButton = {
            val weight = weightInput.toFloatOrNull()
            val pairs = pairCountInput.toIntOrNull()
            Button(
                onClick = { if (weight != null && pairs != null) onAdd(weight, name.trim(), selectedColor, pairs) },
                enabled = name.isNotBlank() && weight != null && weight > 0f && pairs != null && pairs > 0,
            ) { Text("Add") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}

private fun colorFromHex(hex: String): Color {
    return Color(android.graphics.Color.parseColor(hex))
}
