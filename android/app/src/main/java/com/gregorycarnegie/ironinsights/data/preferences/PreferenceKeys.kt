package com.gregorycarnegie.ironinsights.data.preferences

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey

object PreferenceKeys {
    val WEIGHT_UNIT = stringPreferencesKey("weight_unit")
    val BAR_WEIGHT_KG = floatPreferencesKey("bar_weight_kg")
    val ROUNDING_INCREMENT = floatPreferencesKey("rounding_increment")
    val PLATE_INVENTORY_ID = longPreferencesKey("plate_inventory_id")
    val BAR_PRESET_ID = longPreferencesKey("bar_preset_id")
    val HEALTH_CONNECT_ENABLED = booleanPreferencesKey("health_connect_enabled")

    // Profile / onboarding
    val ONBOARDING_COMPLETED = booleanPreferencesKey("onboarding_completed")
    val SEX = stringPreferencesKey("profile_sex")
    val BODYWEIGHT_KG = floatPreferencesKey("profile_bodyweight_kg")
    val HEIGHT_CM = floatPreferencesKey("profile_height_cm")
    val AGE = intPreferencesKey("profile_age")
    val EQUIPMENT = stringPreferencesKey("profile_equipment")
    val TESTED = stringPreferencesKey("profile_tested")
    val SQUAT_KG = floatPreferencesKey("profile_squat_kg")
    val BENCH_KG = floatPreferencesKey("profile_bench_kg")
    val DEADLIFT_KG = floatPreferencesKey("profile_deadlift_kg")
}
