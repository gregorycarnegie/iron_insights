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
