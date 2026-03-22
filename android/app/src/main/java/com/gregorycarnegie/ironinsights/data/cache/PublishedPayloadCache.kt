package com.gregorycarnegie.ironinsights.data.cache

import android.content.Context
import java.io.File

class PublishedPayloadCache(context: Context) {
    private val rootDir = File(context.filesDir, CACHE_DIR_NAME).apply {
        mkdirs()
    }

    fun readText(relativePath: String): String? {
        return readBytes(relativePath)?.toString(Charsets.UTF_8)
    }

    fun writeText(
        relativePath: String,
        value: String,
    ) {
        writeBytes(relativePath, value.toByteArray(Charsets.UTF_8))
    }

    fun readBytes(relativePath: String): ByteArray? {
        val file = fileFor(relativePath) ?: return null
        if (!file.exists() || !file.isFile) {
            return null
        }
        return runCatching { file.readBytes() }.getOrNull()
    }

    fun hasEntry(relativePath: String): Boolean {
        val file = fileFor(relativePath) ?: return false
        return file.exists() && file.isFile
    }

    fun writeBytes(
        relativePath: String,
        value: ByteArray,
    ) {
        val file = fileFor(relativePath) ?: return
        val parent = file.parentFile ?: return
        parent.mkdirs()

        val tempFile = File(parent, "${file.name}.tmp")
        if (tempFile.exists()) {
            tempFile.delete()
        }
        tempFile.writeBytes(value)

        if (file.exists()) {
            file.delete()
        }

        if (!tempFile.renameTo(file)) {
            tempFile.copyTo(file, overwrite = true)
            tempFile.delete()
        }
    }

    fun delete(relativePath: String) {
        val file = fileFor(relativePath) ?: return
        if (file.exists()) {
            file.delete()
        }
    }

    fun cachedVersions(): List<String> {
        return rootDir
            .listFiles()
            ?.asSequence()
            ?.filter { it.isDirectory }
            ?.map { it.name }
            ?.toList()
            ?.let(::normalizeCachedVersions)
            ?: emptyList()
    }

    fun pruneCachedVersions(maxVersions: Int): List<String> {
        val versionsToRemove = versionsToPrune(cachedVersions(), maxVersions)
        versionsToRemove.forEach { version ->
            val directory = fileFor(version) ?: return@forEach
            if (directory.exists() && directory.isDirectory) {
                directory.deleteRecursively()
            }
        }
        return versionsToRemove
    }

    private fun fileFor(relativePath: String): File? {
        val segments = relativePath
            .replace('\\', '/')
            .split('/')
            .filter { it.isNotBlank() }

        if (segments.isEmpty() || segments.any { it == "." || it == ".." }) {
            return null
        }

        return segments.fold(rootDir) { directory, segment ->
            File(directory, segment)
        }
    }

    private companion object {
        const val CACHE_DIR_NAME = "published_payloads"
    }
}

internal fun versionsToPrune(
    cachedVersions: List<String>,
    maxVersions: Int,
): List<String> {
    val normalizedVersions = normalizeCachedVersions(cachedVersions)

    return when {
        maxVersions <= 0 -> normalizedVersions
        normalizedVersions.size <= maxVersions -> emptyList()
        else -> normalizedVersions.drop(maxVersions)
    }
}

private fun normalizeCachedVersions(cachedVersions: List<String>): List<String> {
    return cachedVersions
        .asSequence()
        .map { it.trim() }
        .filter { it.isNotEmpty() }
        .distinct()
        .sortedDescending()
        .toList()
}
