package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition

@Composable
fun ExercisePickerSheet(
    exercises: List<ExerciseDefinition>,
    searchQuery: String,
    onSearchQueryChange: (String) -> Unit,
    onExerciseSelected: (Long) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val filteredExercises = if (searchQuery.isBlank()) {
        exercises
    } else {
        exercises.filter { it.name.contains(searchQuery, ignoreCase = true) }
    }

    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp),
        color = MaterialTheme.colorScheme.surface,
        tonalElevation = 3.dp,
    ) {
        Column {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Add Exercise",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f),
                )
                TextButton(onClick = onDismiss) {
                    Text(
                        text = "Close",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
            OutlinedTextField(
                value = searchQuery,
                onValueChange = onSearchQueryChange,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 4.dp),
                placeholder = { Text("Search exercises") },
                singleLine = true,
            )
            HorizontalDivider(
                modifier = Modifier.padding(top = 8.dp),
                color = MaterialTheme.colorScheme.outlineVariant,
            )
            LazyColumn {
                items(filteredExercises, key = { it.id }) { exercise ->
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { onExerciseSelected(exercise.id) }
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                    ) {
                        Text(
                            text = exercise.name,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = "${exercise.category} \u2022 ${exercise.muscleGroup}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                }
            }
        }
    }
}
