package com.gregorycarnegie.ironinsights.data.cache

import android.content.Context

class LatestDatasetVersionCache(context: Context) {
    private val preferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    fun read(): String? = preferences.getString(KEY_LATEST_VERSION, null)

    fun write(version: String) {
        preferences.edit().putString(KEY_LATEST_VERSION, version).apply()
    }

    fun clear() {
        preferences.edit().remove(KEY_LATEST_VERSION).apply()
    }

    private companion object {
        const val PREFS_NAME = "iron_insights_latest_dataset"
        const val KEY_LATEST_VERSION = "latest_dataset_version"
    }
}
