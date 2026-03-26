package com.gregorycarnegie.ironinsights.ui.progress

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.data.repository.TrainingRepository
import com.gregorycarnegie.ironinsights.domain.progress.E1rmTrendBuilder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch

class ProgressViewModel(
    private val repository: TrainingRepository,
    private val database: IronInsightsDatabase,
) : ViewModel() {

    var uiState by mutableStateOf(ProgressUiState(isLoading = true))
        private set

    private val exerciseDefDao = database.exerciseDefinitionDao()
    private val setEntryDao = database.setEntryDao()

    init {
        loadProgressData()
    }

    fun refresh() {
        loadProgressData()
    }

    private fun loadProgressData() {
        viewModelScope.launch(Dispatchers.IO) {
            uiState = uiState.copy(isLoading = true)

            val canonicalLifts = listOf("S", "B", "D", "T")
            val trends = mutableListOf<E1rmTrendBuilder.E1rmTrend>()
            val prMap = mutableMapOf<String, PrSummary>()

            for (lift in canonicalLifts) {
                val exercises = exerciseDefDao.getByCanonicalLift(lift).firstOrNull()
                    ?: continue

                for (exercise in exercises) {
                    val historyEntries = setEntryDao.getE1rmHistory(exercise.id).firstOrNull()
                        ?: continue

                    val setDataList = historyEntries.map { entry ->
                        E1rmTrendBuilder.SetData(
                            completedAtEpochMs = entry.completedAtEpochMs ?: 0L,
                            e1rmKg = entry.e1rmKg,
                            weightKg = entry.weightKg,
                            reps = entry.reps,
                        )
                    }

                    val trend = E1rmTrendBuilder.buildTrend(
                        exerciseId = exercise.id,
                        exerciseName = exercise.name,
                        canonicalLift = exercise.canonicalLift,
                        sets = setDataList,
                    )

                    if (trend.points.isNotEmpty()) {
                        trends.add(trend)
                    }

                    val prEntries = setEntryDao.getPersonalRecords(exercise.id).firstOrNull()
                    val bestPr = prEntries?.maxByOrNull { it.e1rmKg ?: 0f }
                    val allTimeMax = setEntryDao.getAllTimeMaxE1rm(exercise.id).firstOrNull()

                    val liftName = when (lift) {
                        "S" -> "Squat"
                        "B" -> "Bench"
                        "D" -> "Deadlift"
                        "T" -> "Total"
                        else -> exercise.name
                    }

                    val existingPr = prMap[lift]
                    val candidateE1rm = allTimeMax ?: bestPr?.e1rmKg
                    if (existingPr == null || (candidateE1rm != null && candidateE1rm > (existingPr.bestE1rmKg ?: 0f))) {
                        prMap[lift] = PrSummary(
                            liftName = liftName,
                            bestE1rmKg = candidateE1rm,
                            bestWeightKg = bestPr?.weightKg,
                            bestReps = bestPr?.reps,
                        )
                    }
                }
            }

            uiState = ProgressUiState(
                trends = trends,
                personalRecords = prMap,
                isLoading = false,
            )
        }
    }

    companion object {
        fun factory(
            repository: TrainingRepository,
            database: IronInsightsDatabase,
        ): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    if (modelClass.isAssignableFrom(ProgressViewModel::class.java)) {
                        return ProgressViewModel(repository, database) as T
                    }
                    throw IllegalArgumentException("Unknown ViewModel class")
                }
            }
    }
}
