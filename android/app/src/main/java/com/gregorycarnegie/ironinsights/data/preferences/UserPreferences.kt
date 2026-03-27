package com.gregorycarnegie.ironinsights.data.preferences

import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit

data class UserPreferences(
    val weightUnit: WeightUnit = WeightUnit.KG,
    val barWeightKg: Float = 20f,
    val roundingIncrement: Float = 2.5f,
    val plateInventoryId: Long? = null,
    val barPresetId: Long? = null,
    val healthConnectEnabled: Boolean = false,

    // Profile / onboarding fields
    val onboardingCompleted: Boolean = false,
    val sex: String = "",
    val bodyweightKg: Float? = null,
    val heightCm: Float? = null,
    val age: Int? = null,
    val equipment: String = "",
    val tested: String = "",
    val squatKg: Float? = null,
    val benchKg: Float? = null,
    val deadliftKg: Float? = null,
)
