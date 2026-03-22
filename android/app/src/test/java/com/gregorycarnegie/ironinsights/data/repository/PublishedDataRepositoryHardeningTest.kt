package com.gregorycarnegie.ironinsights.data.repository

import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSource
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PublishedDataRepositoryHardeningTest {
    @Test
    fun resolveLatestFallbackResource_prefers_version_cache_when_root_index_exists() {
        val result = resolveLatestFallbackResource(
            preferredVersion = "2026-03-20",
            cachedVersions = listOf("2026-03-19", "2026-03-18"),
            hasVersionRootIndex = { version -> version == "2026-03-20" },
        )

        requireNotNull(result)
        assertEquals("2026-03-20", result.value.version)
        assertEquals(DatasetLoadSource.VERSION_CACHE, result.source)
    }

    @Test
    fun resolveLatestFallbackResource_falls_back_to_newest_usable_cached_version_when_pointer_is_stale() {
        val result = resolveLatestFallbackResource(
            preferredVersion = "2026-03-21",
            cachedVersions = listOf("2026-03-18", "2026-03-20", "2026-03-19"),
            hasVersionRootIndex = { version -> version == "2026-03-20" || version == "2026-03-19" },
        )

        requireNotNull(result)
        assertEquals("2026-03-20", result.value.version)
        assertEquals(DatasetLoadSource.DISK_CACHE, result.source)
    }

    @Test
    fun resolveLatestFallbackResource_returns_null_when_no_usable_cached_version_exists() {
        val result = resolveLatestFallbackResource(
            preferredVersion = "2026-03-21",
            cachedVersions = listOf("2026-03-20", "2026-03-19"),
            hasVersionRootIndex = { false },
        )

        assertNull(result)
    }
}
