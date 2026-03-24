package com.gregorycarnegie.ironinsights.ui.programmes

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks

@Composable
fun ProgrammeDetailScreen(
    programmeWithBlocks: ProgrammeWithBlocks?,
    onAddBlock: (String, String, Int) -> Unit,
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

    Scaffold(modifier = modifier) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(background),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
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
                Text(
                    text = programmeWithBlocks?.programme?.name ?: "Programme",
                    style = MaterialTheme.typography.headlineMedium,
                    color = MaterialTheme.colorScheme.onBackground,
                    fontWeight = FontWeight.Bold,
                )
                programmeWithBlocks?.programme?.notes?.let { notes ->
                    Text(
                        text = notes,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            val blocks = programmeWithBlocks?.blocks?.sortedBy { it.orderIndex } ?: emptyList()

            items(blocks, key = { it.id }) { block ->
                BlockCard(
                    block = block,
                    onDelete = { onDeleteBlock(block) },
                )
            }

            item {
                OutlinedButton(
                    onClick = {
                        val nextIndex = blocks.size
                        onAddBlock("Block ${nextIndex + 1}", "hypertrophy", 4)
                    },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                ) {
                    Text("+ Add Block")
                }
            }

            item { Spacer(modifier = Modifier.height(32.dp)) }
        }
    }
}

@Composable
private fun BlockCard(
    block: ProgrammeBlock,
    onDelete: () -> Unit,
) {
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
                Column {
                    Text(
                        text = block.name,
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = "${block.blockType} \u2022 ${block.weekCount} weeks",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                TextButton(onClick = onDelete) {
                    Text(
                        text = "Remove",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
        }
    }
}
