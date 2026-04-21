package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.ExercisePerformed
import com.gregorycarnegie.ironinsights.data.db.relation.ExerciseWithSets
import kotlinx.coroutines.flow.Flow

@Dao
interface ExercisePerformedDao {
    @Transaction
    @Query("SELECT * FROM exercises_performed WHERE sessionId = :sessionId ORDER BY orderIndex ASC")
    fun getBySessionId(sessionId: Long): Flow<List<ExerciseWithSets>>

    @Query("SELECT COALESCE(MAX(orderIndex), -1) + 1 FROM exercises_performed WHERE sessionId = :sessionId")
    suspend fun getNextOrderIndex(sessionId: Long): Int

    @Insert
    suspend fun insert(exercise: ExercisePerformed): Long

    @Update
    suspend fun update(exercise: ExercisePerformed)

    @Delete
    suspend fun delete(exercise: ExercisePerformed)
}
