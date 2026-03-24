package com.gregorycarnegie.ironinsights.ui.log

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.util.Locale

@Composable
fun RestTimerBar(
    remainingSeconds: Int,
    isRunning: Boolean,
    onStop: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!isRunning) return

    val minutes = remainingSeconds / 60
    val seconds = remainingSeconds % 60
    val timeText = String.format(Locale.US, "%02d:%02d", minutes, seconds)

    Surface(
        modifier = modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.primaryContainer,
        tonalElevation = 2.dp,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Rest",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Text(
                text = timeText,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            TextButton(onClick = onStop) {
                Text(
                    text = "Stop",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                )
            }
        }
    }
}
