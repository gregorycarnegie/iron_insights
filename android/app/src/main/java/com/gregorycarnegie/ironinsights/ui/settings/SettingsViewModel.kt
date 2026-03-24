package com.gregorycarnegie.ironinsights.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class SettingsViewModel(
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    val uiState: StateFlow<SettingsUiState> = preferencesRepository.preferencesFlow
        .map { preferences -> SettingsUiState(preferences = preferences) }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5_000),
            initialValue = SettingsUiState(),
        )

    fun updateWeightUnit(unit: WeightUnit) {
        viewModelScope.launch {
            preferencesRepository.updateWeightUnit(unit)
        }
    }

    fun updateBarWeight(kg: Float) {
        viewModelScope.launch {
            preferencesRepository.updateBarWeight(kg)
        }
    }

    fun updateRoundingIncrement(increment: Float) {
        viewModelScope.launch {
            preferencesRepository.updateRoundingIncrement(increment)
        }
    }

    fun updatePlateInventoryId(id: Long?) {
        viewModelScope.launch {
            preferencesRepository.updatePlateInventoryId(id)
        }
    }

    fun updateHealthConnectEnabled(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.updateHealthConnectEnabled(enabled)
        }
    }

    companion object {
        fun factory(preferencesRepository: UserPreferencesRepository): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return SettingsViewModel(preferencesRepository) as T
                }
            }
        }
    }
}
