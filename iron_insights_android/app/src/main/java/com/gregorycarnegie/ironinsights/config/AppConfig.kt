package com.gregorycarnegie.ironinsights.config

import com.gregorycarnegie.ironinsights.BuildConfig

data class EnvironmentConfig(
    val siteBaseUrl: String,
    val dataBaseUrl: String,
)

object AppConfig {
    val environment: EnvironmentConfig by lazy {
        val siteBaseUrl = BuildConfig.SITE_BASE_URL.ensureTrailingSlash()
        EnvironmentConfig(
            siteBaseUrl = siteBaseUrl,
            dataBaseUrl = "${siteBaseUrl}data/",
        )
    }

    fun resolvePublishedPath(relativePath: String): String {
        val normalized = relativePath.trimStart('/')
        return environment.siteBaseUrl + normalized
    }
}

private fun String.ensureTrailingSlash(): String = if (endsWith("/")) this else "$this/"
