package com.gregorycarnegie.ironinsights.data.export

import android.content.Context
import android.net.Uri
import androidx.work.CoroutineWorker
import androidx.work.Data
import androidx.work.WorkerParameters
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession
import com.gregorycarnegie.ironinsights.domain.export.StrongCsvParser
import kotlinx.coroutines.flow.first
import java.text.SimpleDateFormat
import java.util.Locale

class CsvImportWorker(
    appContext: Context,
    params: WorkerParameters,
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        return try {
            val uriString = inputData.getString(KEY_FILE_URI)
                ?: return Result.failure(
                    Data.Builder()
                        .putString(KEY_ERROR, "No file URI provided")
                        .build(),
                )

            val uri = Uri.parse(uriString)
            val csvContent = applicationContext.contentResolver
                .openInputStream(uri)
                ?.bufferedReader()
                ?.use { it.readText() }
                ?: return Result.failure(
                    Data.Builder()
                        .putString(KEY_ERROR, "Could not read file")
                        .build(),
                )

            val parsedRows = StrongCsvParser.parse(csvContent)
            if (parsedRows.isEmpty()) {
                return Result.failure(
                    Data.Builder()
                        .putString(KEY_ERROR, "No valid data found in CSV")
                        .build(),
                )
            }

            val db = IronInsightsDatabase.getInstance(applicationContext)
            val sessionDao = db.workoutSessionDao()
            val exercisePerformedDao = db.exercisePerformedDao()
            val setEntryDao = db.setEntryDao()
            val exerciseDefDao = db.exerciseDefinitionDao()

            val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US)

            // Cache existing exercise definitions by name (case-insensitive)
            val allExercises = exerciseDefDao.getAll().first()
            val exerciseByName = allExercises.associateBy { it.name.lowercase() }.toMutableMap()

            // Group parsed rows by date + workout name to form sessions
            val grouped = parsedRows.groupBy { "${it.date}||${it.workoutName}" }

            var sessionsImported = 0
            var setsImported = 0

            for ((_, rows) in grouped) {
                val firstRow = rows.first()

                // Parse the session date
                val sessionDate = try {
                    dateFormat.parse(firstRow.date)?.time ?: System.currentTimeMillis()
                } catch (_: Exception) {
                    System.currentTimeMillis()
                }

                // Create a new session
                val sessionId = sessionDao.insert(
                    WorkoutSession(
                        startedAtEpochMs = sessionDate,
                        finishedAtEpochMs = sessionDate,
                        title = firstRow.workoutName,
                    ),
                )
                sessionsImported++

                // Group rows by exercise name within this session
                val exerciseGroups = rows.groupBy { it.exerciseName }
                var orderIndex = 0

                for ((exerciseName, exerciseRows) in exerciseGroups) {
                    // Find or create exercise definition
                    val exerciseId = exerciseByName[exerciseName.lowercase()]?.id
                        ?: run {
                            val newDef = ExerciseDefinition(
                                name = exerciseName,
                                isBuiltIn = false,
                            )
                            val id = exerciseDefDao.insert(newDef)
                            exerciseByName[exerciseName.lowercase()] = newDef.copy(id = id)
                            id
                        }

                    // Create the ExercisePerformed record
                    val exercisePerformedId = exercisePerformedDao.insert(
                        ExercisePerformed(
                            sessionId = sessionId,
                            exerciseId = exerciseId,
                            orderIndex = orderIndex,
                            notes = exerciseRows.firstOrNull()?.notes,
                        ),
                    )
                    orderIndex++

                    // Create set entries
                    val setEntries = exerciseRows.mapIndexed { index, row ->
                        SetEntry(
                            exercisePerformedId = exercisePerformedId,
                            setIndex = index,
                            weightKg = row.weight,
                            reps = row.reps,
                            rpe = row.rpe,
                            completedAtEpochMs = sessionDate,
                        )
                    }
                    setEntryDao.insertAll(setEntries)
                    setsImported += setEntries.size
                }
            }

            Result.success(
                Data.Builder()
                    .putInt(KEY_SESSIONS_IMPORTED, sessionsImported)
                    .putInt(KEY_SETS_IMPORTED, setsImported)
                    .build(),
            )
        } catch (e: Exception) {
            Result.failure(
                Data.Builder()
                    .putString(KEY_ERROR, e.message ?: "Unknown error")
                    .build(),
            )
        }
    }

    companion object {
        const val KEY_FILE_URI = "file_uri"
        const val KEY_SESSIONS_IMPORTED = "sessions_imported"
        const val KEY_SETS_IMPORTED = "sets_imported"
        const val KEY_ERROR = "error"
    }
}
