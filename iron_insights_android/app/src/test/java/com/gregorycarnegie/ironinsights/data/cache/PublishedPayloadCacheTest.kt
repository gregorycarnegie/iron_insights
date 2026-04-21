package com.gregorycarnegie.ironinsights.data.cache

import com.gregorycarnegie.ironinsights.data.repository.extractVersionSegment
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PublishedPayloadCacheTest {
    @Test
    fun versionsToPrune_keeps_the_newest_versions_only() {
        val result = versionsToPrune(
            cachedVersions = listOf("2026-03-18", "2026-03-20", "2026-03-19"),
            maxVersions = 2,
        )

        assertEquals(listOf("2026-03-18"), result)
    }

    @Test
    fun versionsToPrune_returns_all_versions_when_limit_is_zero() {
        val result = versionsToPrune(
            cachedVersions = listOf("2026-03-19", "2026-03-20"),
            maxVersions = 0,
        )

        assertEquals(listOf("2026-03-20", "2026-03-19"), result)
    }

    @Test
    fun extractVersionSegment_returns_null_for_latest_pointer_path() {
        assertNull(extractVersionSegment("latest.json"))
    }

    @Test
    fun extractVersionSegment_returns_the_version_directory_for_versioned_payloads() {
        assertEquals(
            "2026-03-20",
            extractVersionSegment("2026-03-20/index_shards/M/Raw/index.json"),
        )
    }
}
