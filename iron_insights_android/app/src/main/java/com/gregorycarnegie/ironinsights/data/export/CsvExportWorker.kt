package com.gregorycarnegie.ironinsights.data.export

import android.content.ContentValues
import android.content.Context
import android.os.Build
import android.provider.MediaStore
import androidx.work.CoroutineWorker
import androidx.work.Data
import androidx.work.WorkerParameters
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.domain.export.IronInsightsCsvWriter
import kotlinx.coroutines.flow.first
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class CsvExportWorker(
    appContext: Context,
    params: WorkerParameters,
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        return try {
            val db = IronInsightsDatabase.getInstance(applicationContext)
            val sessionDao = db.workoutSessionDao()
            val exercisePerformedDao = db.exercisePerformedDao()
            val exerciseDefDao = db.exerciseDefinitionDao()
            val setEntryDao = db.setEntryDao()

            val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US)

            // Get all sessions
            val sessions = sessionDao.getRecentSessions(Int.MAX_VALUE).first()

            val exportSessions = sessions.map { session ->
                val exercisesWithSets = exercisePerformedDao
                    .getBySessionId(session.id)
                    .first()

                val exportExercises = exercisesWithSets.map { ews ->
                    val definition = exerciseDefDao.getById(ews.exercise.exerciseId)
                    val exportSets = ews.sets
                        .sortedBy { it.setIndex }
                        .mapIndexed { index, set ->
                            IronInsightsCsvWriter.ExportSet(
                                setOrder = index + 1,
                                weightKg = set.weightKg,
                                reps = set.reps,
                                rpe = set.rpe,
                                notes = ews.exercise.notes,
                            )
                        }
                    IronInsightsCsvWriter.ExportExercise(
                        name = definition?.name ?: "Unknown Exercise",
                        sets = exportSets,
                    )
                }

                IronInsightsCsvWriter.ExportSession(
                    date = dateFormat.format(Date(session.startedAtEpochMs)),
                    workoutName = session.title ?: "Workout",
                    exercises = exportExercises,
                )
            }

            val csvContent = IronInsightsCsvWriter.write(exportSessions)

            val fileTimestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US)
                .format(Date())
            val fileName = "iron_insights_export_$fileTimestamp.csv"

            val uri = saveToDownloads(fileName, csvContent, ExportFormat.CSV)
                ?: return Result.failure(
                    Data.Builder()
                        .putString(KEY_ERROR, "Failed to save file")
                        .build(),
                )

            Result.success(
                Data.Builder()
                    .putString(KEY_FILE_URI, uri.toString())
                    .putString(KEY_FILE_NAME, fileName)
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

    private fun saveToDownloads(
        fileName: String,
        content: String,
        format: ExportFormat,
    ): android.net.Uri? {
        val resolver = applicationContext.contentResolver
        val contentValues = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, fileName)
            put(MediaStore.Downloads.MIME_TYPE, format.mimeType)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.Downloads.IS_PENDING, 1)
            }
        }

        val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
            ?: return null

        resolver.openOutputStream(uri)?.use { outputStream ->
            outputStream.write(content.toByteArray(Charsets.UTF_8))
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            contentValues.clear()
            contentValues.put(MediaStore.Downloads.IS_PENDING, 0)
            resolver.update(uri, contentValues, null, null)
        }

        return uri
    }

    companion object {
        const val KEY_FILE_URI = "file_uri"
        const val KEY_FILE_NAME = "file_name"
        const val KEY_ERROR = "error"
    }
}
