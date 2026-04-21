package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "planned_sessions",
    foreignKeys = [ForeignKey(entity = ProgrammeBlock::class, parentColumns = ["id"], childColumns = ["blockId"], onDelete = ForeignKey.CASCADE)],
    indices = [Index("blockId")],
)
data class PlannedSession(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val blockId: Long,
    val dayOfWeek: Int? = null,
    val weekIndex: Int = 0,
    val title: String? = null,
    val notes: String? = null,
)
