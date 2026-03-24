package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.SetEntry
import kotlinx.coroutines.flow.Flow

@Dao
interface SetEntryDao {
    @Query("SELECT * FROM set_entries WHERE exercisePerformedId = :exercisePerformedId ORDER BY setIndex ASC")
    fun getByExercisePerformedId(exercisePerformedId: Long): Flow<List<SetEntry>>

    @Insert
    suspend fun insert(set: SetEntry): Long

    @Insert
    suspend fun insertAll(sets: List<SetEntry>)

    @Update
    suspend fun update(set: SetEntry)

    @Delete
    suspend fun delete(set: SetEntry)

    @Query("""
        SELECT se.* FROM set_entries se
        INNER JOIN exercises_performed ep ON se.exercisePerformedId = ep.id
        WHERE ep.exerciseId = :exerciseId AND se.isPersonalRecord = 1
        ORDER BY se.e1rmKg DESC
    """)
    fun getPersonalRecords(exerciseId: Long): Flow<List<SetEntry>>

    @Query("""
        SELECT se.* FROM set_entries se
        INNER JOIN exercises_performed ep ON se.exercisePerformedId = ep.id
        WHERE ep.exerciseId = :exerciseId AND se.e1rmKg IS NOT NULL AND se.isWarmup = 0
        ORDER BY se.completedAtEpochMs ASC
    """)
    fun getE1rmHistory(exerciseId: Long): Flow<List<SetEntry>>

    @Query("""
        SELECT MAX(se.e1rmKg) FROM set_entries se
        INNER JOIN exercises_performed ep ON se.exercisePerformedId = ep.id
        WHERE ep.exerciseId = :exerciseId AND se.isWarmup = 0
    """)
    fun getAllTimeMaxE1rm(exerciseId: Long): Flow<Float?>
}
