package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedExercise
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSession
import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet
import com.gregorycarnegie.ironinsights.data.db.relation.PlannedSessionWithExercises
import kotlinx.coroutines.flow.Flow

@Dao
interface PlannedSessionDao {
    @Transaction
    @Query("SELECT * FROM planned_sessions WHERE blockId = :blockId ORDER BY weekIndex ASC, dayOfWeek ASC")
    fun getByBlockId(blockId: Long): Flow<List<PlannedSessionWithExercises>>

    @Insert
    suspend fun insertSession(session: PlannedSession): Long

    @Update
    suspend fun updateSession(session: PlannedSession)

    @Delete
    suspend fun deleteSession(session: PlannedSession)

    @Insert
    suspend fun insertExercise(exercise: PlannedExercise): Long

    @Update
    suspend fun updateExercise(exercise: PlannedExercise)

    @Delete
    suspend fun deleteExercise(exercise: PlannedExercise)

    @Insert
    suspend fun insertSet(set: PlannedSet): Long

    @Update
    suspend fun updateSet(set: PlannedSet)

    @Delete
    suspend fun deleteSet(set: PlannedSet)
}
