package com.gregorycarnegie.ironinsights.ui.equipment

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.db.dao.EquipmentDao
import com.gregorycarnegie.ironinsights.data.db.entity.BarPreset
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventory
import com.gregorycarnegie.ironinsights.data.db.entity.PlateInventoryItem
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

data class EquipmentUiState(
    val inventories: List<PlateInventory> = emptyList(),
    val selectedInventoryItems: List<PlateInventoryItem> = emptyList(),
    val selectedInventoryId: Long? = null,
    val barPresets: List<BarPreset> = emptyList(),
    val activeInventoryId: Long? = null,
    val activeBarPresetId: Long? = null,
    val isLoading: Boolean = true,
)

class EquipmentViewModel(
    private val equipmentDao: EquipmentDao,
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    var uiState by mutableStateOf(EquipmentUiState())
        private set

    init {
        loadInventories()
        loadBarPresets()
        loadPreferences()
    }

    private fun loadInventories() {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.getAllInventories().collect { inventories ->
                uiState = uiState.copy(inventories = inventories, isLoading = false)
            }
        }
    }

    private fun loadBarPresets() {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.getAllBarPresets().collect { presets ->
                uiState = uiState.copy(barPresets = presets)
            }
        }
    }

    private fun loadPreferences() {
        viewModelScope.launch(Dispatchers.IO) {
            preferencesRepository.preferencesFlow.collect { prefs ->
                uiState = uiState.copy(
                    activeInventoryId = prefs.plateInventoryId,
                    activeBarPresetId = prefs.barPresetId,
                )
            }
        }
    }

    fun selectInventory(inventoryId: Long) {
        uiState = uiState.copy(selectedInventoryId = inventoryId)
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.getItemsByInventoryId(inventoryId).collect { items ->
                uiState = uiState.copy(selectedInventoryItems = items)
            }
        }
    }

    fun clearSelection() {
        uiState = uiState.copy(selectedInventoryId = null, selectedInventoryItems = emptyList())
    }

    fun createInventory(name: String) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.insertInventory(PlateInventory(name = name))
        }
    }

    fun deleteInventory(inventory: PlateInventory) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.deleteInventory(inventory)
            if (uiState.selectedInventoryId == inventory.id) {
                uiState = uiState.copy(selectedInventoryId = null, selectedInventoryItems = emptyList())
            }
            if (uiState.activeInventoryId == inventory.id) {
                preferencesRepository.updatePlateInventoryId(null)
            }
        }
    }

    fun addItem(inventoryId: Long, weightKg: Float, name: String, colorHex: String, pairCount: Int) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.insertItem(
                PlateInventoryItem(
                    inventoryId = inventoryId,
                    weightKg = weightKg,
                    name = name,
                    colorHex = colorHex,
                    pairCount = pairCount,
                ),
            )
        }
    }

    fun updateItem(item: PlateInventoryItem) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.updateItem(item)
        }
    }

    fun deleteItem(item: PlateInventoryItem) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.deleteItem(item)
        }
    }

    fun setActiveInventory(inventoryId: Long?) {
        viewModelScope.launch(Dispatchers.IO) {
            preferencesRepository.updatePlateInventoryId(inventoryId)
        }
    }

    fun createBarPreset(name: String, weightKg: Float) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.insertBarPreset(BarPreset(name = name, weightKg = weightKg))
        }
    }

    fun deleteBarPreset(preset: BarPreset) {
        viewModelScope.launch(Dispatchers.IO) {
            equipmentDao.deleteBarPreset(preset)
            if (uiState.activeBarPresetId == preset.id) {
                preferencesRepository.updateBarPresetId(null)
            }
        }
    }

    fun setActiveBarPreset(presetId: Long?) {
        viewModelScope.launch(Dispatchers.IO) {
            preferencesRepository.updateBarPresetId(presetId)
        }
    }

    companion object {
        fun factory(
            equipmentDao: EquipmentDao,
            preferencesRepository: UserPreferencesRepository,
        ): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return EquipmentViewModel(equipmentDao, preferencesRepository) as T
                }
            }
        }
    }
}
