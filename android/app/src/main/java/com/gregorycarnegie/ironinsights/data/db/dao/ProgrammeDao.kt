package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.*
import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import com.gregorycarnegie.ironinsights.data.db.relation.ProgrammeWithBlocks
import kotlinx.coroutines.flow.Flow

@Dao
interface ProgrammeDao {
    @Query("SELECT * FROM programmes ORDER BY createdAtEpochMs DESC")
    fun getAll(): Flow<List<Programme>>

    @Transaction
    @Query("SELECT * FROM programmes WHERE id = :id")
    fun getById(id: Long): Flow<ProgrammeWithBlocks?>

    @Query("SELECT * FROM programmes WHERE isActive = 1 LIMIT 1")
    fun getActive(): Flow<Programme?>

    @Insert
    suspend fun insertProgramme(programme: Programme): Long

    @Update
    suspend fun updateProgramme(programme: Programme)

    @Delete
    suspend fun deleteProgramme(programme: Programme)

    @Insert
    suspend fun insertBlock(block: ProgrammeBlock): Long

    @Update
    suspend fun updateBlock(block: ProgrammeBlock)

    @Delete
    suspend fun deleteBlock(block: ProgrammeBlock)

    @Query("SELECT * FROM programme_blocks WHERE programmeId = :programmeId ORDER BY orderIndex ASC")
    fun getBlocksForProgramme(programmeId: Long): Flow<List<ProgrammeBlock>>
}
