package com.gregorycarnegie.ironinsights.data.health

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.domain.health.WorkoutToHealthRecord
import kotlinx.coroutines.flow.firstOrNull

class HealthConnectSyncWorker(
    context: Context,
    params: WorkerParameters,
) : CoroutineWorker(context, params) {

    override suspend fun doWork(): Result {
        val sessionId = inputData.getLong("session_id", -1L)
        if (sessionId == -1L) return Result.failure()

        val manager = HealthConnectManager(applicationContext)
        if (!manager.isAvailable() || !manager.hasAllPermissions()) {
            return Result.failure()
        }

        val database = IronInsightsDatabase.getInstance(applicationContext)
        val sessionWithExercises = database.workoutSessionDao().getById(sessionId).firstOrNull()
            ?: return Result.failure()

        val record = WorkoutToHealthRecord.convert(sessionWithExercises.session)
            ?: return Result.failure()

        return try {
            manager.writeExerciseSession(record)
            Result.success()
        } catch (e: Exception) {
            Result.retry()
        }
    }
}
