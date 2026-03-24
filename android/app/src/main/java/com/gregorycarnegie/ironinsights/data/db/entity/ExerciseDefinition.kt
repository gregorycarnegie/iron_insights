package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "exercise_definitions")
data class ExerciseDefinition(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val name: String,
    val canonicalLift: String? = null,    // "S", "B", "D", "T" — maps to Iron Insights dataset lift codes
    val category: String = "compound",    // compound, isolation, accessory
    val muscleGroup: String = "legs",
    val isBuiltIn: Boolean = true,
    val isArchived: Boolean = false,
)
