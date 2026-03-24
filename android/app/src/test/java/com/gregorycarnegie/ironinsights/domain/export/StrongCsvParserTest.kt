package com.gregorycarnegie.ironinsights.domain.export

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class StrongCsvParserTest {

    @Test
    fun parse_valid_csv() {
        val csv = """
            Date,Workout Name,Exercise Name,Set Order,Weight,Reps,RPE,Notes
            2026-03-23,Morning,Squat,1,140,5,8,
            2026-03-23,Morning,Squat,2,140,5,8.5,felt heavy
        """.trimIndent()
        val result = StrongCsvParser.parse(csv)
        assertEquals(2, result.size)
        assertEquals("Squat", result[0].exerciseName)
        assertEquals(140f, result[0].weight, 0.01f)
        assertEquals(5, result[0].reps)
        assertEquals(8f, result[0].rpe)
        assertNull(result[0].notes)
        assertEquals("felt heavy", result[1].notes)
    }

    @Test
    fun parse_handles_empty_fields() {
        val csv = """
            Date,Workout Name,Exercise Name,Set Order,Weight,Reps,,
            2026-03-23,Morning,Bench,1,100,5,,
        """.trimIndent()
        val result = StrongCsvParser.parse(csv)
        assertEquals(1, result.size)
        assertNull(result[0].rpe)
        assertNull(result[0].notes)
    }

    @Test
    fun parse_handles_quoted_fields_with_commas() {
        val csv = """
            Date,Workout Name,Exercise Name,Set Order,Weight,Reps,RPE,Notes
            2026-03-23,Morning,"Bench Press, Close Grip",1,80,8,,
        """.trimIndent()
        val result = StrongCsvParser.parse(csv)
        assertEquals(1, result.size)
        assertEquals("Bench Press, Close Grip", result[0].exerciseName)
    }

    @Test
    fun parse_empty_csv_returns_empty() {
        assertEquals(emptyList<StrongCsvParser.ParsedWorkout>(), StrongCsvParser.parse(""))
    }
}
