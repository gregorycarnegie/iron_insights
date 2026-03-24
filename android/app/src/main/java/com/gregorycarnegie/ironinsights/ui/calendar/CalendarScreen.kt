package com.gregorycarnegie.ironinsights.ui.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock

@Composable
fun CalendarScreen(
    uiState: CalendarUiState,
    modifier: Modifier = Modifier,
) {
    val background = Brush.verticalGradient(
        colors = listOf(
            MaterialTheme.colorScheme.background,
            MaterialTheme.colorScheme.surface,
        ),
    )

    Scaffold(modifier = modifier) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(background)
                .padding(16.dp),
        ) {
            Text(
                text = "Training Calendar",
                style = MaterialTheme.typography.headlineMedium,
                color = MaterialTheme.colorScheme.onBackground,
                fontWeight = FontWeight.Bold,
            )

            if (uiState.blocks.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(32.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "No active programme. Create a programme and set it as active to see your training calendar.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.padding(top = 16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    items(uiState.blocks, key = { it.id }) { block ->
                        BlockTimelineCard(block = block)
                    }
                }
            }
        }
    }
}

@Composable
private fun BlockTimelineCard(block: ProgrammeBlock) {
    val blockColor = when (block.blockType) {
        "hypertrophy" -> MaterialTheme.colorScheme.secondary
        "strength" -> MaterialTheme.colorScheme.primary
        "peak" -> MaterialTheme.colorScheme.error
        "deload" -> MaterialTheme.colorScheme.tertiary
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .height(40.dp)
                    .background(
                        color = blockColor,
                        shape = RoundedCornerShape(4.dp),
                    )
                    .padding(horizontal = 8.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "${block.weekCount}w",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onPrimary,
                )
            }
            Column {
                Text(
                    text = block.name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    fontWeight = FontWeight.SemiBold,
                )
                Text(
                    text = block.blockType.replaceFirstChar { it.uppercase() },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
