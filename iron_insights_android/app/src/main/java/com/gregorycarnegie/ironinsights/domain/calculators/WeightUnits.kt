package com.gregorycarnegie.ironinsights.domain.calculators

import java.util.Locale
import kotlin.math.abs
import kotlin.math.round

enum class WeightUnit(
    val label: String,
) {
    KG("kg"),
    LB("lb"),
}

private const val KG_TO_LB = 2.2046225f

fun kgToDisplay(
    kg: Float,
    unit: WeightUnit,
): Float {
    return if (unit == WeightUnit.LB) kg * KG_TO_LB else kg
}

fun displayToKg(
    value: Float,
    unit: WeightUnit,
): Float {
    return if (unit == WeightUnit.LB) value / KG_TO_LB else value
}

fun formatWeightInput(value: Float): String {
    return when {
        abs(value - round(value)) < 0.05f -> String.format(Locale.US, "%.0f", value)
        abs((value * 2f) - round(value * 2f)) < 0.05f -> String.format(Locale.US, "%.1f", value)
        else -> String.format(Locale.US, "%.2f", value)
    }
}

fun formatInputBound(
    valueKg: Float,
    unit: WeightUnit,
): String {
    return formatWeightInput(kgToDisplay(valueKg, unit))
}
