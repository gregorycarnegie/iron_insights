package com.gregorycarnegie.ironinsights.data.db.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "plate_inventory_items",
    foreignKeys = [
        ForeignKey(
            entity = PlateInventory::class,
            parentColumns = ["id"],
            childColumns = ["inventoryId"],
            onDelete = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("inventoryId")],
)
data class PlateInventoryItem(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val inventoryId: Long,
    val weightKg: Float,
    val name: String,
    val colorHex: String,
    val pairCount: Int = 1,
)
