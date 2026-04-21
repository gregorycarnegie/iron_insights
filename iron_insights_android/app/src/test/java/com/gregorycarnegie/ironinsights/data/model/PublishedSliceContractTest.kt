package com.gregorycarnegie.ironinsights.data.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PublishedSliceContractTest {
    @Test
    fun parseSliceKey_defaultsMetricToKg() {
        val key =
            PublishedSliceContract.parseSliceKey("sex=M|equip=Raw|wc=93|age=Open|tested=Yes|lift=T")

        requireNotNull(key)
        assertEquals("M", key.sex)
        assertEquals("Raw", key.equip)
        assertEquals("93", key.wc)
        assertEquals("Open", key.age)
        assertEquals("Yes", key.tested)
        assertEquals("T", key.lift)
        assertEquals("Kg", key.metric)
        assertFalse(key.metricExplicit)
    }

    @Test
    fun parseSliceKey_rejectsMalformedSegments() {
        assertNull(
            PublishedSliceContract.parseSliceKey(
                "sex=M|equip=Raw|broken|wc=93|age=Open|tested=Yes|lift=T",
            ),
        )
    }

    @Test
    fun parseSliceKey_rejectsEmptyValues() {
        assertNull(
            PublishedSliceContract.parseSliceKey(
                "sex=M|equip=Raw|wc=|age=Open|tested=Yes|lift=T",
            ),
        )
    }

    @Test
    fun parseShardKey_extractsSexAndEquip() {
        val key = PublishedSliceContract.parseShardKey("equip=Raw|sex=F|ignored=value")

        requireNotNull(key)
        assertEquals("F", key.sex)
        assertEquals("Raw", key.equip)
    }

    @Test
    fun entryFromSliceKey_omitsMetricDirectoryForLegacyKeys() {
        val entry =
            PublishedSliceContract.entryFromSliceKey("sex=M|equip=Raw|wc=All|age=All Ages|tested=Yes|lift=T")

        requireNotNull(entry)
        assertFalse(entry.key.metricExplicit)
        assertEquals("meta/m/raw/all/all_ages/tested/total.json", entry.paths.meta)
        assertEquals("bin/m/raw/all/all_ages/tested/total.bin", entry.paths.bin)
    }

    @Test
    fun entryFromSliceKey_includesMetricDirectoryWhenExplicit() {
        val entry = PublishedSliceContract.entryFromSliceKey(
            "sex=F|equip=Raw|wc=63|age=Open|tested=All|lift=B|metric=Lb",
        )

        requireNotNull(entry)
        assertTrue(entry.key.metricExplicit)
        assertEquals("meta/f/raw/63/open/all/lb/bench.json", entry.paths.meta)
        assertEquals("bin/f/raw/63/open/all/lb/bench.bin", entry.paths.bin)
    }
}
