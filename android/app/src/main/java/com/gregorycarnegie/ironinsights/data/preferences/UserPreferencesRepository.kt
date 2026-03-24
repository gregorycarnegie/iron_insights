package com.gregorycarnegie.ironinsights.data.preferences

import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class UserPreferencesRepository(
    private val dataStore: DataStore<Preferences>,
) {
    val preferencesFlow: Flow<UserPreferences> = dataStore.data.map { preferences ->
        val weightUnitRaw = preferences[PreferenceKeys.WEIGHT_UNIT]
        val weightUnit = WeightUnit.entries.firstOrNull { it.name == weightUnitRaw } ?: WeightUnit.KG
        val barWeightKg = preferences[PreferenceKeys.BAR_WEIGHT_KG] ?: 20f
        val roundingIncrement = preferences[PreferenceKeys.ROUNDING_INCREMENT] ?: 2.5f
        val plateInventoryId = preferences[PreferenceKeys.PLATE_INVENTORY_ID]
        val barPresetId = preferences[PreferenceKeys.BAR_PRESET_ID]
        val healthConnectEnabled = preferences[PreferenceKeys.HEALTH_CONNECT_ENABLED] ?: false

        UserPreferences(
            weightUnit = weightUnit,
            barWeightKg = barWeightKg,
            roundingIncrement = roundingIncrement,
            plateInventoryId = plateInventoryId,
            barPresetId = barPresetId,
            healthConnectEnabled = healthConnectEnabled,
        )
    }

    suspend fun updateWeightUnit(unit: WeightUnit) {
        dataStore.edit { preferences ->
            preferences[PreferenceKeys.WEIGHT_UNIT] = unit.name
        }
    }

    suspend fun updateBarWeight(kg: Float) {
        dataStore.edit { preferences ->
            preferences[PreferenceKeys.BAR_WEIGHT_KG] = kg
        }
    }

    suspend fun updateRoundingIncrement(increment: Float) {
        dataStore.edit { preferences ->
            preferences[PreferenceKeys.ROUNDING_INCREMENT] = increment
        }
    }

    suspend fun updatePlateInventoryId(id: Long?) {
        dataStore.edit { preferences ->
            if (id != null) {
                preferences[PreferenceKeys.PLATE_INVENTORY_ID] = id
            } else {
                preferences.remove(PreferenceKeys.PLATE_INVENTORY_ID)
            }
        }
    }

    suspend fun updateBarPresetId(id: Long?) {
        dataStore.edit { preferences ->
            if (id != null) {
                preferences[PreferenceKeys.BAR_PRESET_ID] = id
            } else {
                preferences.remove(PreferenceKeys.BAR_PRESET_ID)
            }
        }
    }

    suspend fun updateHealthConnectEnabled(enabled: Boolean) {
        dataStore.edit { preferences ->
            preferences[PreferenceKeys.HEALTH_CONNECT_ENABLED] = enabled
        }
    }
}
