package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.Dao
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface TrainingStatsDao {
    @Query("""
        SELECT COALESCE(SUM(se.weightKg * se.reps), 0) FROM set_entries se
        INNER JOIN exercises_performed ep ON se.exercisePerformedId = ep.id
        INNER JOIN workout_sessions ws ON ep.sessionId = ws.id
        WHERE ep.exerciseId = :exerciseId
          AND ws.startedAtEpochMs BETWEEN :startMs AND :endMs
          AND se.isWarmup = 0
    """)
    fun getVolumeLoad(exerciseId: Long, startMs: Long, endMs: Long): Flow<Float>

    @Query("""
        SELECT se.e1rmKg FROM set_entries se
        INNER JOIN exercises_performed ep ON se.exercisePerformedId = ep.id
        WHERE ep.exerciseId = :exerciseId AND se.e1rmKg IS NOT NULL AND se.isWarmup = 0
        ORDER BY se.completedAtEpochMs DESC
        LIMIT 1
    """)
    fun getLatestE1rm(exerciseId: Long): Flow<Float?>
}
