package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition
import kotlinx.coroutines.flow.Flow

@Dao
interface ExerciseDefinitionDao {
    @Query("SELECT * FROM exercise_definitions WHERE isArchived = 0 ORDER BY name ASC")
    fun getAll(): Flow<List<ExerciseDefinition>>

    @Query("SELECT * FROM exercise_definitions WHERE canonicalLift = :lift AND isArchived = 0")
    fun getByCanonicalLift(lift: String): Flow<List<ExerciseDefinition>>

    @Query("SELECT * FROM exercise_definitions WHERE id = :id")
    suspend fun getById(id: Long): ExerciseDefinition?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(exercise: ExerciseDefinition): Long

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(exercises: List<ExerciseDefinition>)

    @Update
    suspend fun update(exercise: ExerciseDefinition)

    @Query("SELECT COUNT(*) FROM exercise_definitions")
    suspend fun count(): Int
}
