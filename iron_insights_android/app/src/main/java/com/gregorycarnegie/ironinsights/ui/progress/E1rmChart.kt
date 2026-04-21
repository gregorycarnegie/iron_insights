package com.gregorycarnegie.ironinsights.ui.progress

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.formatWeightInput
import com.gregorycarnegie.ironinsights.domain.calculators.kgToDisplay
import com.gregorycarnegie.ironinsights.domain.progress.E1rmTrendBuilder.E1rmTrend
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import kotlin.math.max
import kotlin.math.min

@Composable
fun E1rmChart(trend: E1rmTrend, weightUnit: WeightUnit, modifier: Modifier = Modifier) {
    if (trend.points.isEmpty()) return

    val points = trend.points
    val displayValues = points.map { kgToDisplay(it.e1rmKg, weightUnit) }
    val minValue = displayValues.min()
    val maxValue = displayValues.max()
    val padding = ((maxValue - minValue) * 0.1f).coerceAtLeast(1f)
    val lower = minValue - padding
    val upper = maxValue + padding
    val valueSpan = (upper - lower).coerceAtLeast(1f)

    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.7f)
    val lineColor = MaterialTheme.colorScheme.primary
    val dotColor = MaterialTheme.colorScheme.primary

    val dateFormat = SimpleDateFormat("MMM yy", Locale.US)

    Surface(
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.72f),
        modifier = modifier,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = trend.exerciseName,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            trend.allTimeBestKg?.let { best ->
                val displayBest = kgToDisplay(best, weightUnit)
                Text(
                    text = "Best: ${formatWeightInput(displayBest)} ${weightUnit.label}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            }

            // Y-axis labels
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = "${formatWeightInput(upper)} ${weightUnit.label}",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "Est. 1RM",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp),
            ) {
                val chartWidth = size.width
                val chartHeight = size.height

                // Grid lines
                repeat(4) { index ->
                    val fraction = index / 3f
                    val y = chartHeight * fraction
                    drawLine(
                        color = gridColor,
                        start = Offset(0f, y),
                        end = Offset(chartWidth, y),
                        strokeWidth = 1.5f,
                    )
                }

                if (points.size == 1) {
                    // Single point: draw a dot in the center
                    val centerX = chartWidth / 2f
                    val y = chartHeight - ((displayValues[0] - lower) / valueSpan) * chartHeight
                    drawCircle(
                        color = dotColor,
                        radius = 6f,
                        center = Offset(centerX, y),
                    )
                } else {
                    val stepX = chartWidth / (points.size - 1).toFloat()

                    // Draw lines
                    for (i in 1 until points.size) {
                        val prevY = chartHeight - ((displayValues[i - 1] - lower) / valueSpan) * chartHeight
                        val currY = chartHeight - ((displayValues[i] - lower) / valueSpan) * chartHeight
                        drawLine(
                            color = lineColor,
                            start = Offset((i - 1) * stepX, prevY),
                            end = Offset(i * stepX, currY),
                            strokeWidth = 4f,
                        )
                    }

                    // Draw dots
                    points.forEachIndexed { index, _ ->
                        val y = chartHeight - ((displayValues[index] - lower) / valueSpan) * chartHeight
                        drawCircle(
                            color = dotColor,
                            radius = 4f,
                            center = Offset(index * stepX, y),
                        )
                    }
                }
            }

            // X-axis labels (dates)
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                val firstDate = dateFormat.format(Date(points.first().epochMs))
                val lastDate = dateFormat.format(Date(points.last().epochMs))
                Text(
                    text = firstDate,
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "${formatWeightInput(lower)} ${weightUnit.label}",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = lastDate,
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
