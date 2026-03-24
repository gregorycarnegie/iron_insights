package com.gregorycarnegie.ironinsights.data.preferences

import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit

data class UserPreferences(
    val weightUnit: WeightUnit = WeightUnit.KG,
    val barWeightKg: Float = 20f,
    val roundingIncrement: Float = 2.5f,
    val plateInventoryId: Long? = null,
    val barPresetId: Long? = null,
    val healthConnectEnabled: Boolean = false,
)
