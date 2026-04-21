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
            onboardingCompleted = preferences[PreferenceKeys.ONBOARDING_COMPLETED] ?: false,
            sex = preferences[PreferenceKeys.SEX] ?: "",
            bodyweightKg = preferences[PreferenceKeys.BODYWEIGHT_KG],
            heightCm = preferences[PreferenceKeys.HEIGHT_CM],
            age = preferences[PreferenceKeys.AGE],
            equipment = preferences[PreferenceKeys.EQUIPMENT] ?: "",
            tested = preferences[PreferenceKeys.TESTED] ?: "",
            squatKg = preferences[PreferenceKeys.SQUAT_KG],
            benchKg = preferences[PreferenceKeys.BENCH_KG],
            deadliftKg = preferences[PreferenceKeys.DEADLIFT_KG],
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

    suspend fun completeOnboarding(
        sex: String,
        bodyweightKg: Float?,
        heightCm: Float?,
        age: Int?,
        equipment: String,
        tested: String,
        squatKg: Float?,
        benchKg: Float?,
        deadliftKg: Float?,
    ) {
        dataStore.edit { prefs ->
            prefs[PreferenceKeys.ONBOARDING_COMPLETED] = true
            prefs[PreferenceKeys.SEX] = sex
            if (bodyweightKg != null) prefs[PreferenceKeys.BODYWEIGHT_KG] = bodyweightKg
            else prefs.remove(PreferenceKeys.BODYWEIGHT_KG)
            if (heightCm != null) prefs[PreferenceKeys.HEIGHT_CM] = heightCm
            else prefs.remove(PreferenceKeys.HEIGHT_CM)
            if (age != null) prefs[PreferenceKeys.AGE] = age
            else prefs.remove(PreferenceKeys.AGE)
            prefs[PreferenceKeys.EQUIPMENT] = equipment
            prefs[PreferenceKeys.TESTED] = tested
            if (squatKg != null) prefs[PreferenceKeys.SQUAT_KG] = squatKg
            else prefs.remove(PreferenceKeys.SQUAT_KG)
            if (benchKg != null) prefs[PreferenceKeys.BENCH_KG] = benchKg
            else prefs.remove(PreferenceKeys.BENCH_KG)
            if (deadliftKg != null) prefs[PreferenceKeys.DEADLIFT_KG] = deadliftKg
            else prefs.remove(PreferenceKeys.DEADLIFT_KG)
        }
    }

}
