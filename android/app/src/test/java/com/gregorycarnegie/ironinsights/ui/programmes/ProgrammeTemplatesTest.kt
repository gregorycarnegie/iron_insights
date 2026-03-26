package com.gregorycarnegie.ironinsights.ui.programmes

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class ProgrammeTemplatesTest {

    @Test
    fun `every block template default weeks fit within recommended range`() {
        programmeBlockTemplates.forEach { template ->
            assertTrue(
                "${template.blockType} default weeks should fit inside ${template.weekRange}",
                template.defaultWeeks in template.weekRange,
            )
        }
    }

    @Test
    fun `traditional meet prep preset totals twelve weeks`() {
        val preset = programmePresetTemplates.first { it.id == "traditional_meet_prep" }
        assertEquals(12, programmePresetTotalWeeks(preset))
        assertEquals(listOf("hypertrophy", "strength", "peak", "taper"), preset.blocks.map { it.blockType })
    }

    @Test
    fun `block peak preset ends with taper`() {
        val preset = programmePresetTemplates.first { it.id == "block_peak_cycle" }
        assertEquals("taper", preset.blocks.last().blockType)
        assertEquals(10, programmePresetTotalWeeks(preset))
    }
}
