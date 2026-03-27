package com.gregorycarnegie.ironinsights.ui.profile

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferences
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

data class ProfileUiState(
    val preferences: UserPreferences = UserPreferences(),
    val isEditing: Boolean = false,
    val sexInput: String = "",
    val bodyweightInput: String = "",
    val heightInput: String = "",
    val ageInput: String = "",
    val equipmentInput: String = "",
    val testedInput: String = "",
    val squatInput: String = "",
    val benchInput: String = "",
    val deadliftInput: String = "",
)

class ProfileViewModel(
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    var uiState by mutableStateOf(ProfileUiState())
        private set

    init {
        viewModelScope.launch {
            preferencesRepository.preferencesFlow.collect { prefs ->
                val unit = prefs.weightUnit
                uiState = uiState.copy(
                    preferences = prefs,
                    sexInput = prefs.sex,
                    bodyweightInput = prefs.bodyweightKg?.let { formatForUnit(it, unit) } ?: "",
                    heightInput = prefs.heightCm?.let { "%.0f".format(it) } ?: "",
                    ageInput = prefs.age?.toString() ?: "",
                    equipmentInput = prefs.equipment,
                    testedInput = prefs.tested,
                    squatInput = prefs.squatKg?.let { formatForUnit(it, unit) } ?: "",
                    benchInput = prefs.benchKg?.let { formatForUnit(it, unit) } ?: "",
                    deadliftInput = prefs.deadliftKg?.let { formatForUnit(it, unit) } ?: "",
                )
            }
        }
    }

    fun startEditing() {
        uiState = uiState.copy(isEditing = true)
    }

    fun cancelEditing() {
        val prefs = uiState.preferences
        val unit = prefs.weightUnit
        uiState = uiState.copy(
            isEditing = false,
            sexInput = prefs.sex,
            bodyweightInput = prefs.bodyweightKg?.let { formatForUnit(it, unit) } ?: "",
            heightInput = prefs.heightCm?.let { "%.0f".format(it) } ?: "",
            ageInput = prefs.age?.toString() ?: "",
            equipmentInput = prefs.equipment,
            testedInput = prefs.tested,
            squatInput = prefs.squatKg?.let { formatForUnit(it, unit) } ?: "",
            benchInput = prefs.benchKg?.let { formatForUnit(it, unit) } ?: "",
            deadliftInput = prefs.deadliftKg?.let { formatForUnit(it, unit) } ?: "",
        )
    }

    fun updateSex(value: String) { uiState = uiState.copy(sexInput = value) }
    fun updateBodyweight(value: String) { uiState = uiState.copy(bodyweightInput = value) }
    fun updateHeight(value: String) { uiState = uiState.copy(heightInput = value) }
    fun updateAge(value: String) { uiState = uiState.copy(ageInput = value) }
    fun updateEquipment(value: String) { uiState = uiState.copy(equipmentInput = value) }
    fun updateTested(value: String) { uiState = uiState.copy(testedInput = value) }
    fun updateSquat(value: String) { uiState = uiState.copy(squatInput = value) }
    fun updateBench(value: String) { uiState = uiState.copy(benchInput = value) }
    fun updateDeadlift(value: String) { uiState = uiState.copy(deadliftInput = value) }

    fun saveProfile() {
        val unit = uiState.preferences.weightUnit
        val bwKg = uiState.bodyweightInput.toFloatOrNull()?.let {
            if (unit == WeightUnit.LB) it / 2.20462f else it
        }
        val heightCm = uiState.heightInput.toFloatOrNull()
        val age = uiState.ageInput.toIntOrNull()
        val squatKg = uiState.squatInput.toFloatOrNull()?.let {
            if (unit == WeightUnit.LB) it / 2.20462f else it
        }
        val benchKg = uiState.benchInput.toFloatOrNull()?.let {
            if (unit == WeightUnit.LB) it / 2.20462f else it
        }
        val deadliftKg = uiState.deadliftInput.toFloatOrNull()?.let {
            if (unit == WeightUnit.LB) it / 2.20462f else it
        }

        viewModelScope.launch(Dispatchers.IO) {
            preferencesRepository.updateProfile(
                sex = uiState.sexInput,
                bodyweightKg = bwKg,
                heightCm = heightCm,
                age = age,
                equipment = uiState.equipmentInput,
                tested = uiState.testedInput,
                squatKg = squatKg,
                benchKg = benchKg,
                deadliftKg = deadliftKg,
            )
            viewModelScope.launch(Dispatchers.Main) {
                uiState = uiState.copy(isEditing = false)
            }
        }
    }

    companion object {
        fun factory(
            preferencesRepository: UserPreferencesRepository,
        ): ViewModelProvider.Factory = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T {
                return ProfileViewModel(preferencesRepository) as T
            }
        }

        private fun formatForUnit(kg: Float, unit: WeightUnit): String {
            val value = if (unit == WeightUnit.LB) kg * 2.20462f else kg
            return if (value == value.toLong().toFloat()) "%.0f".format(value)
            else "%.1f".format(value)
        }
    }
}
