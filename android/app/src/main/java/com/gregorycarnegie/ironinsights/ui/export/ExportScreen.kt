package com.gregorycarnegie.ironinsights.ui.export

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun ExportScreen(
    viewModel: ExportViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val state = viewModel.uiState

    val csvImportLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent(),
    ) { uri: Uri? ->
        uri?.let { viewModel.importCsv(it, context) }
    }

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
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                TextButton(onClick = onNavigateBack) {
                    Text(
                        text = "\u2190 Back",
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            item {
                Text(
                    text = "Export & Import",
                    style = MaterialTheme.typography.headlineMedium,
                    color = MaterialTheme.colorScheme.onBackground,
                    fontWeight = FontWeight.Bold,
                )
            }

            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant,
                    ),
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            text = "Export Training Data",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                            fontWeight = FontWeight.SemiBold,
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Export your workouts to a file in your Downloads folder.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Button(
                            onClick = { viewModel.exportCsv(context) },
                            enabled = !state.isExporting,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                            ),
                            shape = RoundedCornerShape(8.dp),
                        ) {
                            Text("Export as CSV (Strong-compatible)")
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedButton(
                            onClick = { viewModel.exportJson(context) },
                            enabled = !state.isExporting,
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(8.dp),
                        ) {
                            Text("Export as JSON (full fidelity)")
                        }
                    }
                }
            }

            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant,
                    ),
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            text = "Import Training Data",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                            fontWeight = FontWeight.SemiBold,
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Import workouts from a Strong-compatible CSV file.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        OutlinedButton(
                            onClick = { csvImportLauncher.launch("text/*") },
                            enabled = !state.isImporting,
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(8.dp),
                        ) {
                            Text("Import from CSV")
                        }
                    }
                }
            }

            if (state.isExporting || state.isImporting) {
                item {
                    CircularProgressIndicator(
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            state.lastResult?.let { result ->
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.surface,
                        ),
                    ) {
                        Text(
                            text = result,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier.padding(16.dp),
                        )
                    }
                }
            }

            item { Spacer(modifier = Modifier.height(32.dp)) }
        }
    }
}
