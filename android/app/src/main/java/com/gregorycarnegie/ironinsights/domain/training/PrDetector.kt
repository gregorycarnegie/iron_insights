package com.gregorycarnegie.ironinsights.domain.training

object PrDetector {
    data class PrResult(
        val isE1rmPr: Boolean,
        val isWeightPr: Boolean,
        val isRepsPr: Boolean,
    ) {
        val isAnyPr: Boolean get() = isE1rmPr || isWeightPr || isRepsPr
    }

    fun detect(
        candidateE1rmKg: Float,
        candidateWeightKg: Float,
        candidateReps: Int,
        previousBestE1rmKg: Float?,
        previousBestWeightKg: Float?,
        previousBestRepsAtOrAboveWeight: Int?,
    ): PrResult {
        val isE1rmPr = previousBestE1rmKg == null || candidateE1rmKg > previousBestE1rmKg
        val isWeightPr = previousBestWeightKg == null || candidateWeightKg > previousBestWeightKg
        val isRepsPr = previousBestRepsAtOrAboveWeight != null && candidateReps > previousBestRepsAtOrAboveWeight
                || previousBestRepsAtOrAboveWeight == null && candidateReps >= 1
        return PrResult(isE1rmPr, isWeightPr, isRepsPr)
    }
}
