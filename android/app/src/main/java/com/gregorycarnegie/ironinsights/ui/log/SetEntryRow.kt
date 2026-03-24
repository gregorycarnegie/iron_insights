package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import java.util.Locale

@Composable
fun SetEntryRow(
    setNumber: Int,
    set: SetEntry,
    weightUnit: WeightUnit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val displayWeight = kgToDisplay(set.weightKg, weightUnit)
    val formattedWeight = formatWeightInput(displayWeight)
    val formattedE1rm = set.e1rmKg?.let {
        formatWeightInput(kgToDisplay(it, weightUnit))
    }

    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = "$setNumber",
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(0.8f),
        )
        Text(
            text = "$formattedWeight ${weightUnit.label}",
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1.5f),
        )
        Text(
            text = "${set.reps}",
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(0.8f),
        )
        Text(
            text = set.rpe?.let { String.format(Locale.US, "%.1f", it) } ?: "-",
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(0.8f),
        )
        Text(
            text = formattedE1rm ?: "-",
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(1.2f),
        )
        TextButton(
            onClick = onDelete,
            modifier = Modifier.size(32.dp),
        ) {
            Text(
                text = "\u00D7",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}
