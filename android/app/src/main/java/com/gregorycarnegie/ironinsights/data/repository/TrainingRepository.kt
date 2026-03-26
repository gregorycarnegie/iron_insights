package com.gregorycarnegie.ironinsights.data.repository

import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession
import com.gregorycarnegie.ironinsights.data.db.relation.SessionWithExercises
import com.gregorycarnegie.ironinsights.domain.calculators.OneRepMaxCalculator
import kotlinx.coroutines.flow.Flow

class TrainingRepository(database: IronInsightsDatabase) {

    private val sessionDao = database.workoutSessionDao()
    private val exercisePerformedDao = database.exercisePerformedDao()
    private val setEntryDao = database.setEntryDao()

    suspend fun startSession(
        title: String? = null,
        notes: String? = null,
        programmeSessionId: Long? = null,
        bodyweightKg: Float? = null,
    ): Long {
        val session = WorkoutSession(
            startedAtEpochMs = System.currentTimeMillis(),
            title = title,
            notes = notes,
            programmeSessionId = programmeSessionId,
            bodyweightKg = bodyweightKg,
        )
        return sessionDao.insert(session)
    }

    suspend fun finishSession(sessionId: Long) {
        sessionDao.finishSession(
            sessionId = sessionId,
            finishedAtEpochMs = System.currentTimeMillis(),
        )
    }

    suspend fun addExercise(
        sessionId: Long,
        exerciseId: Long,
        notes: String? = null,
    ): Long {
        val nextIndex = exercisePerformedDao.getNextOrderIndex(sessionId)
        val exercisePerformed = ExercisePerformed(
            sessionId = sessionId,
            exerciseId = exerciseId,
            orderIndex = nextIndex,
            notes = notes,
        )
        return exercisePerformedDao.insert(exercisePerformed)
    }

    suspend fun logSet(
        exercisePerformedId: Long,
        weightKg: Float,
        reps: Int,
        rpe: Float? = null,
        isWarmup: Boolean = false,
    ): Long {
        val nextSetIndex = setEntryDao.getNextSetIndex(exercisePerformedId)
        val e1rmKg = if (!isWarmup && reps > 0) {
            OneRepMaxCalculator.blended1rm(weightKg, reps)
        } else {
            null
        }
        val setEntry = SetEntry(
            exercisePerformedId = exercisePerformedId,
            setIndex = nextSetIndex,
            weightKg = weightKg,
            reps = reps,
            rpe = rpe,
            isWarmup = isWarmup,
            e1rmKg = e1rmKg,
            completedAtEpochMs = System.currentTimeMillis(),
        )
        return setEntryDao.insert(setEntry)
    }

    suspend fun deleteSet(set: SetEntry) {
        setEntryDao.delete(set)
    }

    suspend fun deleteExercise(exercise: ExercisePerformed) {
        exercisePerformedDao.delete(exercise)
    }

    fun getActiveSession(): Flow<WorkoutSession?> {
        return sessionDao.getActiveSession()
    }

    fun getSessionWithExercises(sessionId: Long): Flow<SessionWithExercises?> {
        return sessionDao.getById(sessionId)
    }

    fun getRecentSessions(limit: Int = 20): Flow<List<WorkoutSession>> {
        return sessionDao.getRecentSessions(limit)
    }

    fun getRecentSessionDetails(limit: Int = 20): Flow<List<SessionWithExercises>> {
        return sessionDao.getRecentSessionDetails(limit)
    }
}
