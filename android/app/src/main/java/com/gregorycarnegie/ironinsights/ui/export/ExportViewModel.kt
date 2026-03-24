package com.gregorycarnegie.ironinsights.ui.export

import android.content.Context
import android.net.Uri
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.work.Data
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkInfo
import com.gregorycarnegie.ironinsights.data.export.CsvExportWorker
import com.gregorycarnegie.ironinsights.data.export.CsvImportWorker
import com.gregorycarnegie.ironinsights.data.export.JsonExportWorker
import kotlinx.coroutines.launch

class ExportViewModel : ViewModel() {

    var uiState by mutableStateOf(ExportUiState())
        private set

    fun exportCsv(context: Context) {
        uiState = uiState.copy(isExporting = true, lastResult = null)
        val request = OneTimeWorkRequestBuilder<CsvExportWorker>().build()
        val workManager = WorkManager.getInstance(context)
        workManager.enqueue(request)

        viewModelScope.launch {
            workManager.getWorkInfoByIdFlow(request.id).collect { workInfo ->
                when (workInfo?.state) {
                    WorkInfo.State.SUCCEEDED -> {
                        uiState = uiState.copy(isExporting = false, lastResult = "CSV exported successfully")
                    }
                    WorkInfo.State.FAILED -> {
                        uiState = uiState.copy(isExporting = false, lastResult = "CSV export failed")
                    }
                    else -> {}
                }
            }
        }
    }

    fun exportJson(context: Context) {
        uiState = uiState.copy(isExporting = true, lastResult = null)
        val request = OneTimeWorkRequestBuilder<JsonExportWorker>().build()
        val workManager = WorkManager.getInstance(context)
        workManager.enqueue(request)

        viewModelScope.launch {
            workManager.getWorkInfoByIdFlow(request.id).collect { workInfo ->
                when (workInfo?.state) {
                    WorkInfo.State.SUCCEEDED -> {
                        uiState = uiState.copy(isExporting = false, lastResult = "JSON exported successfully")
                    }
                    WorkInfo.State.FAILED -> {
                        uiState = uiState.copy(isExporting = false, lastResult = "JSON export failed")
                    }
                    else -> {}
                }
            }
        }
    }

    fun importCsv(uri: Uri, context: Context) {
        uiState = uiState.copy(isImporting = true, lastResult = null)
        val inputData = Data.Builder()
            .putString("uri", uri.toString())
            .build()
        val request = OneTimeWorkRequestBuilder<CsvImportWorker>()
            .setInputData(inputData)
            .build()
        val workManager = WorkManager.getInstance(context)
        workManager.enqueue(request)

        viewModelScope.launch {
            workManager.getWorkInfoByIdFlow(request.id).collect { workInfo ->
                when (workInfo?.state) {
                    WorkInfo.State.SUCCEEDED -> {
                        val count = workInfo.outputData.getInt("imported_count", 0)
                        uiState = uiState.copy(isImporting = false, lastResult = "Imported $count workouts")
                    }
                    WorkInfo.State.FAILED -> {
                        uiState = uiState.copy(isImporting = false, lastResult = "Import failed")
                    }
                    else -> {}
                }
            }
        }
    }

    companion object {
        fun factory(): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return ExportViewModel() as T
                }
            }
        }
    }
}
