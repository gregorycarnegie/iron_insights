package com.gregorycarnegie.ironinsights.data.preferences

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey

object PreferenceKeys {
    val WEIGHT_UNIT = stringPreferencesKey("weight_unit")
    val BAR_WEIGHT_KG = floatPreferencesKey("bar_weight_kg")
    val ROUNDING_INCREMENT = floatPreferencesKey("rounding_increment")
    val PLATE_INVENTORY_ID = longPreferencesKey("plate_inventory_id")
    val BAR_PRESET_ID = longPreferencesKey("bar_preset_id")
    val HEALTH_CONNECT_ENABLED = booleanPreferencesKey("health_connect_enabled")
}
