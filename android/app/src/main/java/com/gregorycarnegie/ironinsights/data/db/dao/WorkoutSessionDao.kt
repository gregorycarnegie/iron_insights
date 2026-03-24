package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.WorkoutSession
import com.gregorycarnegie.ironinsights.data.db.relation.SessionWithExercises
import kotlinx.coroutines.flow.Flow

@Dao
interface WorkoutSessionDao {
    @Transaction
    @Query("SELECT * FROM workout_sessions WHERE id = :id")
    fun getById(id: Long): Flow<SessionWithExercises?>

    @Query("SELECT * FROM workout_sessions ORDER BY startedAtEpochMs DESC LIMIT :limit")
    fun getRecentSessions(limit: Int = 20): Flow<List<WorkoutSession>>

    @Query("SELECT * FROM workout_sessions WHERE startedAtEpochMs BETWEEN :startMs AND :endMs ORDER BY startedAtEpochMs DESC")
    fun getSessionsBetween(startMs: Long, endMs: Long): Flow<List<WorkoutSession>>

    @Query("SELECT * FROM workout_sessions WHERE finishedAtEpochMs IS NULL ORDER BY startedAtEpochMs DESC LIMIT 1")
    fun getActiveSession(): Flow<WorkoutSession?>

    @Insert
    suspend fun insert(session: WorkoutSession): Long

    @Update
    suspend fun update(session: WorkoutSession)

    @Delete
    suspend fun delete(session: WorkoutSession)
}
