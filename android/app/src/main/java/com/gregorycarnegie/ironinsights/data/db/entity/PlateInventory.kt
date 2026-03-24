package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "plate_inventories")
data class PlateInventory(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val name: String,
    val isDefault: Boolean = false,
)
