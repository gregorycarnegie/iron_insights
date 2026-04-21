package com.gregorycarnegie.ironinsights.ui.onboarding

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.domain.calculators.displayToKg
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class OnboardingUiState(
    val step: OnboardingStep = OnboardingStep.BODY_METRICS,
    val sex: String = "M",
    val bodyweightInput: String = "",
    val heightInput: String = "",
    val ageInput: String = "",
    val equipment: String = "Raw",
    val tested: String = "All",
    val squatInput: String = "",
    val benchInput: String = "",
    val deadliftInput: String = "",
    val weightUnit: WeightUnit = WeightUnit.KG,
    val isSaving: Boolean = false,
)

enum class OnboardingStep {
    BODY_METRICS,
    LIFT_NUMBERS,
}

class OnboardingViewModel(
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    var uiState by mutableStateOf(OnboardingUiState())
        private set

    init {
        viewModelScope.launch {
            preferencesRepository.preferencesFlow.collect { prefs ->
                uiState = uiState.copy(weightUnit = prefs.weightUnit)
            }
        }
    }

    fun updateSex(value: String) {
        uiState = uiState.copy(sex = value)
    }

    fun updateBodyweight(value: String) {
        uiState = uiState.copy(bodyweightInput = value)
    }

    fun updateHeight(value: String) {
        uiState = uiState.copy(heightInput = value)
    }

    fun updateAge(value: String) {
        uiState = uiState.copy(ageInput = value)
    }

    fun updateEquipment(value: String) {
        uiState = uiState.copy(equipment = value)
    }

    fun updateTested(value: String) {
        uiState = uiState.copy(tested = value)
    }

    fun updateSquat(value: String) {
        uiState = uiState.copy(squatInput = value)
    }

    fun updateBench(value: String) {
        uiState = uiState.copy(benchInput = value)
    }

    fun updateDeadlift(value: String) {
        uiState = uiState.copy(deadliftInput = value)
    }

    fun nextStep() {
        if (uiState.step == OnboardingStep.BODY_METRICS) {
            uiState = uiState.copy(step = OnboardingStep.LIFT_NUMBERS)
        }
    }

    fun previousStep() {
        if (uiState.step == OnboardingStep.LIFT_NUMBERS) {
            uiState = uiState.copy(step = OnboardingStep.BODY_METRICS)
        }
    }

    fun finish(onComplete: () -> Unit) {
        if (uiState.isSaving) return
        uiState = uiState.copy(isSaving = true)

        val unit = uiState.weightUnit
        val bwKg = uiState.bodyweightInput.toFloatOrNull()?.let { displayToKg(it, unit) }
        val heightCm = uiState.heightInput.toFloatOrNull()
        val age = uiState.ageInput.toIntOrNull()
        val squatKg = uiState.squatInput.toFloatOrNull()?.let { displayToKg(it, unit) }
        val benchKg = uiState.benchInput.toFloatOrNull()?.let { displayToKg(it, unit) }
        val deadliftKg = uiState.deadliftInput.toFloatOrNull()?.let { displayToKg(it, unit) }

        viewModelScope.launch(Dispatchers.IO) {
            preferencesRepository.completeOnboarding(
                sex = uiState.sex,
                bodyweightKg = bwKg,
                heightCm = heightCm,
                age = age,
                equipment = uiState.equipment,
                tested = uiState.tested,
                squatKg = squatKg,
                benchKg = benchKg,
                deadliftKg = deadliftKg,
            )
            withContext(Dispatchers.Main) {
                onComplete()
            }
        }
    }

    companion object {
        fun factory(
            preferencesRepository: UserPreferencesRepository,
        ): ViewModelProvider.Factory = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T {
                return OnboardingViewModel(preferencesRepository) as T
            }
        }
    }
}
