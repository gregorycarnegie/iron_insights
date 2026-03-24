package com.gregorycarnegie.ironinsights.ui.export

data class ExportUiState(
    val isExporting: Boolean = false,
    val isImporting: Boolean = false,
    val lastResult: String? = null,
)
