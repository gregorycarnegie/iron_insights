package com.gregorycarnegie.ironinsights.data.export

import android.content.ContentValues
import android.content.Context
import android.os.Build
import android.provider.MediaStore
import androidx.work.CoroutineWorker
import androidx.work.Data
import androidx.work.WorkerParameters
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import kotlinx.coroutines.flow.first
import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class JsonExportWorker(
    appContext: Context,
    params: WorkerParameters,
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result {
        return try {
            val db = IronInsightsDatabase.getInstance(applicationContext)
            val sessionDao = db.workoutSessionDao()
            val exercisePerformedDao = db.exercisePerformedDao()
            val exerciseDefDao = db.exerciseDefinitionDao()

            val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US)

            val sessions = sessionDao.getRecentSessions(Int.MAX_VALUE).first()

            val sessionsArray = JSONArray()
            for (session in sessions) {
                val sessionObj = JSONObject().apply {
                    put("date", dateFormat.format(Date(session.startedAtEpochMs)))
                    put("workoutName", session.title ?: "Workout")
                    put("notes", session.notes ?: JSONObject.NULL)
                    put("bodyweightKg", session.bodyweightKg?.toDouble() ?: JSONObject.NULL)
                    session.finishedAtEpochMs?.let {
                        put("finishedAt", dateFormat.format(Date(it)))
                    }
                }

                val exercisesWithSets = exercisePerformedDao
                    .getBySessionId(session.id)
                    .first()

                val exercisesArray = JSONArray()
                for (ews in exercisesWithSets) {
                    val definition = exerciseDefDao.getById(ews.exercise.exerciseId)
                    val exerciseObj = JSONObject().apply {
                        put("name", definition?.name ?: "Unknown Exercise")
                        put("category", definition?.category ?: "")
                        put("muscleGroup", definition?.muscleGroup ?: "")
                        put("notes", ews.exercise.notes ?: JSONObject.NULL)
                    }

                    val setsArray = JSONArray()
                    for (set in ews.sets.sortedBy { it.setIndex }) {
                        setsArray.put(
                            JSONObject().apply {
                                put("setIndex", set.setIndex)
                                put("weightKg", set.weightKg.toDouble())
                                put("reps", set.reps)
                                put("rpe", set.rpe?.toDouble() ?: JSONObject.NULL)
                                put("rir", set.rir ?: JSONObject.NULL)
                                put("isWarmup", set.isWarmup)
                                put("isPersonalRecord", set.isPersonalRecord)
                                put("e1rmKg", set.e1rmKg?.toDouble() ?: JSONObject.NULL)
                            },
                        )
                    }
                    exerciseObj.put("sets", setsArray)
                    exercisesArray.put(exerciseObj)
                }
                sessionObj.put("exercises", exercisesArray)
                sessionsArray.put(sessionObj)
            }

            val root = JSONObject().apply {
                put("exportedAt", dateFormat.format(Date()))
                put("appVersion", "Iron Insights")
                put("sessions", sessionsArray)
            }

            val jsonContent = root.toString(2)

            val fileTimestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US)
                .format(Date())
            val fileName = "iron_insights_export_$fileTimestamp.json"

            val uri = saveToDownloads(fileName, jsonContent)
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
    ): android.net.Uri? {
        val resolver = applicationContext.contentResolver
        val contentValues = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, fileName)
            put(MediaStore.Downloads.MIME_TYPE, ExportFormat.JSON.mimeType)
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
