package com.gregorycarnegie.ironinsights.domain.progress

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class E1rmTrendBuilderTest {

    @Test
    fun emptySets_returnsEmptyTrend() {
        val trend = E1rmTrendBuilder.buildTrend(
            exerciseId = 1L,
            exerciseName = "Squat",
            canonicalLift = "S",
            sets = emptyList(),
        )
        assertEquals(0, trend.points.size)
        assertNull(trend.currentBestKg)
        assertNull(trend.allTimeBestKg)
        assertEquals(1L, trend.exerciseId)
        assertEquals("Squat", trend.exerciseName)
        assertEquals("S", trend.canonicalLift)
    }

    @Test
    fun singleSetWithE1rm_producesSinglePoint() {
        val trend = E1rmTrendBuilder.buildTrend(
            exerciseId = 2L,
            exerciseName = "Bench Press",
            canonicalLift = "B",
            sets = listOf(
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 1000L,
                    e1rmKg = 100f,
                    weightKg = 80f,
                    reps = 5,
                ),
            ),
        )
        assertEquals(1, trend.points.size)
        assertEquals(100f, trend.points[0].e1rmKg, 0.001f)
        assertEquals(80f, trend.points[0].weightKg, 0.001f)
        assertEquals(5, trend.points[0].reps)
        assertEquals(1000L, trend.points[0].epochMs)
        assertEquals(100f, trend.currentBestKg!!, 0.001f)
        assertEquals(100f, trend.allTimeBestKg!!, 0.001f)
    }

    @Test
    fun setsWithoutE1rm_areFilteredOut() {
        val trend = E1rmTrendBuilder.buildTrend(
            exerciseId = 3L,
            exerciseName = "Deadlift",
            canonicalLift = "D",
            sets = listOf(
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 1000L,
                    e1rmKg = null,
                    weightKg = 60f,
                    reps = 10,
                ),
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 2000L,
                    e1rmKg = 150f,
                    weightKg = 120f,
                    reps = 3,
                ),
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 3000L,
                    e1rmKg = null,
                    weightKg = 40f,
                    reps = 20,
                ),
            ),
        )
        assertEquals(1, trend.points.size)
        assertEquals(150f, trend.points[0].e1rmKg, 0.001f)
    }

    @Test
    fun allTimeBest_isComputedCorrectly() {
        val trend = E1rmTrendBuilder.buildTrend(
            exerciseId = 4L,
            exerciseName = "Squat",
            canonicalLift = "S",
            sets = listOf(
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 1000L,
                    e1rmKg = 100f,
                    weightKg = 80f,
                    reps = 5,
                ),
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 2000L,
                    e1rmKg = 130f,
                    weightKg = 100f,
                    reps = 5,
                ),
                E1rmTrendBuilder.SetData(
                    completedAtEpochMs = 3000L,
                    e1rmKg = 120f,
                    weightKg = 95f,
                    reps = 5,
                ),
            ),
        )
        assertEquals(3, trend.points.size)
        // All-time best is the highest e1rm across all points
        assertEquals(130f, trend.allTimeBestKg!!, 0.001f)
        // Current best is the last point (sorted by time)
        assertEquals(120f, trend.currentBestKg!!, 0.001f)
        // Points should be sorted by epochMs
        assertEquals(1000L, trend.points[0].epochMs)
        assertEquals(2000L, trend.points[1].epochMs)
        assertEquals(3000L, trend.points[2].epochMs)
    }
}
