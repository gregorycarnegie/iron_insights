package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "bar_presets")
data class BarPreset(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val name: String,
    val weightKg: Float,
    val isDefault: Boolean = false,
)
