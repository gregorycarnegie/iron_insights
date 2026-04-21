package com.gregorycarnegie.ironinsights.domain.training

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PrDetectorTest {

    @Test
    fun firstEverSet_isAlwaysPr() {
        val result = PrDetector.detect(
            candidateE1rmKg = 100f,
            candidateWeightKg = 80f,
            candidateReps = 5,
            previousBestE1rmKg = null,
            previousBestWeightKg = null,
            previousBestRepsAtOrAboveWeight = null,
        )
        assertTrue(result.isE1rmPr)
        assertTrue(result.isWeightPr)
        assertTrue(result.isRepsPr)
        assertTrue(result.isAnyPr)
    }

    @Test
    fun higherE1rm_triggersE1rmPr_notWeightOrRepsPr() {
        val result = PrDetector.detect(
            candidateE1rmKg = 120f,
            candidateWeightKg = 80f,
            candidateReps = 5,
            previousBestE1rmKg = 110f,
            previousBestWeightKg = 100f,
            previousBestRepsAtOrAboveWeight = 8,
        )
        assertTrue(result.isE1rmPr)
        assertFalse(result.isWeightPr)
        assertFalse(result.isRepsPr)
        assertTrue(result.isAnyPr)
    }

    @Test
    fun higherWeight_triggersWeightPr() {
        val result = PrDetector.detect(
            candidateE1rmKg = 100f,
            candidateWeightKg = 105f,
            candidateReps = 3,
            previousBestE1rmKg = 110f,
            previousBestWeightKg = 100f,
            previousBestRepsAtOrAboveWeight = 5,
        )
        assertFalse(result.isE1rmPr)
        assertTrue(result.isWeightPr)
        assertFalse(result.isRepsPr)
        assertTrue(result.isAnyPr)
    }

    @Test
    fun moreRepsAtSameWeight_triggersRepsPr() {
        val result = PrDetector.detect(
            candidateE1rmKg = 100f,
            candidateWeightKg = 100f,
            candidateReps = 8,
            previousBestE1rmKg = 110f,
            previousBestWeightKg = 100f,
            previousBestRepsAtOrAboveWeight = 5,
        )
        assertFalse(result.isE1rmPr)
        assertFalse(result.isWeightPr)
        assertTrue(result.isRepsPr)
        assertTrue(result.isAnyPr)
    }

    @Test
    fun noPr_whenAllValuesAreLower() {
        val result = PrDetector.detect(
            candidateE1rmKg = 90f,
            candidateWeightKg = 80f,
            candidateReps = 3,
            previousBestE1rmKg = 100f,
            previousBestWeightKg = 100f,
            previousBestRepsAtOrAboveWeight = 5,
        )
        assertFalse(result.isE1rmPr)
        assertFalse(result.isWeightPr)
        assertFalse(result.isRepsPr)
        assertFalse(result.isAnyPr)
    }
}
