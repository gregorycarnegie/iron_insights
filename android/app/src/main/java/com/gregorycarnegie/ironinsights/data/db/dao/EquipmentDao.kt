package com.gregorycarnegie.ironinsights.data.db.dao

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.gregorycarnegie.ironinsights.data.db.entity.BarPreset
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventory
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventoryItem
import kotlinx.coroutines.flow.Flow

@Dao
interface EquipmentDao {
    @Query("SELECT * FROM plate_inventories ORDER BY name ASC")
    fun getAllInventories(): Flow<List<PlateInventory>>

    @Query("SELECT * FROM plate_inventory_items WHERE inventoryId = :inventoryId ORDER BY weightKg DESC")
    fun getItemsByInventoryId(inventoryId: Long): Flow<List<PlateInventoryItem>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertInventory(inventory: PlateInventory): Long

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertItem(item: PlateInventoryItem): Long

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertItems(items: List<PlateInventoryItem>)

    @Update
    suspend fun updateInventory(inventory: PlateInventory)

    @Update
    suspend fun updateItem(item: PlateInventoryItem)

    @Delete
    suspend fun deleteInventory(inventory: PlateInventory)

    @Delete
    suspend fun deleteItem(item: PlateInventoryItem)

    @Query("SELECT * FROM bar_presets ORDER BY weightKg ASC")
    fun getAllBarPresets(): Flow<List<BarPreset>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertBarPreset(preset: BarPreset): Long

    @Update
    suspend fun updateBarPreset(preset: BarPreset)

    @Delete
    suspend fun deleteBarPreset(preset: BarPreset)
}
