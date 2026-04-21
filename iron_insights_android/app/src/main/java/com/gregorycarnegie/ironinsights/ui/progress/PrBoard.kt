package com.gregorycarnegie.ironinsights.ui.progress

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay

@Composable
fun PrBoard(records: Map<String, PrSummary>, weightUnit: WeightUnit) {
    val liftOrder = listOf("S" to "Squat", "B" to "Bench", "D" to "Deadlift", "T" to "Total")

    Column(
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        liftOrder.chunked(2).forEach { rowItems ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                rowItems.forEach { (code, fallbackName) ->
                    val pr = records[code]
                    PrCard(
                        modifier = Modifier.weight(1f),
                        liftName = pr?.liftName ?: fallbackName,
                        bestE1rmKg = pr?.bestE1rmKg,
                        bestWeightKg = pr?.bestWeightKg,
                        bestReps = pr?.bestReps,
                        percentileLabel = pr?.percentileLabel,
                        weightUnit = weightUnit,
                    )
                }
                if (rowItems.size == 1) {
                    Spacer(modifier = Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun PrCard(
    modifier: Modifier = Modifier,
    liftName: String,
    bestE1rmKg: Float?,
    bestWeightKg: Float?,
    bestReps: Int?,
    percentileLabel: String?,
    weightUnit: WeightUnit,
) {
    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
        shape = RoundedCornerShape(16.dp),
        modifier = modifier,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = liftName,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (bestE1rmKg != null) {
                val displayE1rm = kgToDisplay(bestE1rmKg, weightUnit)
                Text(
                    text = "${formatWeightInput(displayE1rm)} ${weightUnit.label}",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Text(
                    text = "Est. 1RM",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            } else {
                Text(
                    text = "--",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                )
            }
            if (bestWeightKg != null && bestReps != null) {
                val displayWeight = kgToDisplay(bestWeightKg, weightUnit)
                Text(
                    text = "${formatWeightInput(displayWeight)} ${weightUnit.label} x $bestReps",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            percentileLabel?.let { label ->
                Text(
                    text = label,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.tertiary,
                )
            }
        }
    }
}
