package com.example.androidMic.ui.utils

import androidx.appcompat.app.AppCompatActivity
import com.example.androidMic.AndroidMicApp


/**
 * Rules: key should be upper case snake case
 * Ex: SAMPLE_RATE
 * The key should match the name in the app state
 */
class PrefManager(private val androidMicApp: AndroidMicApp) {

    companion object {
        private const val PREFERENCES_NAME = "AndroidMicUserSettingsV2"
    }

    operator fun <T> set(key: String, value: T) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        val editor = userSettings.edit()

        when (value) {
            is Boolean -> {
                editor.putBoolean(key, (value as Boolean))
            }

            is String -> {
                editor.putString(key, value as String)
            }

            is Float -> {
                editor.putFloat(key, (value as Float))
            }

            is Long -> {
                editor.putLong(key, (value as Long))
            }

            is Int -> {
                editor.putInt(key, (value as Int))
            }
        }
        editor.apply()
    }


    /**
     * Warning: Unchecked cast, this could lead to runtime exceptions
     */
    @Suppress("UNCHECKED_CAST")
    operator fun <T> get(key: String, defaultValue: T): T {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        when (defaultValue) {
            is Boolean -> {
                return userSettings.getBoolean(key, (defaultValue as Boolean)) as T
            }


            is String -> {
                return userSettings.getString(key, defaultValue as String) as T
            }

            is Float -> {
                return userSettings.getFloat(key, (defaultValue as Float)) as T
            }

            is Long -> {
                return userSettings.getLong(key, (defaultValue as Long)) as T
            }

            is Int -> {
                return userSettings.getInt(key, (defaultValue as Int)) as T
            }

            else -> return defaultValue
        }
    }
}