package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "programme_blocks",
    foreignKeys = [ForeignKey(entity = Programme::class, parentColumns = ["id"], childColumns = ["programmeId"], onDelete = ForeignKey.CASCADE)],
    indices = [Index("programmeId")],
)
data class ProgrammeBlock(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val programmeId: Long,
    val name: String,
    val blockType: String,     // "hypertrophy", "strength", "peak", "deload"
    val orderIndex: Int,
    val weekCount: Int,
    val startDateEpochMs: Long? = null,
)
