package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "programmes")
data class Programme(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val name: String,
    val notes: String? = null,
    val isActive: Boolean = false,
    val createdAtEpochMs: Long,
)
