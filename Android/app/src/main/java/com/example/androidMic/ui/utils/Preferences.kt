package com.example.androidMic.ui.utils

import androidx.appcompat.app.AppCompatActivity
import com.example.androidMic.AndroidMicApp
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.SampleRates
import com.example.androidMic.ui.Themes


class Preferences(private val androidMicApp: AndroidMicApp) {

    companion object {
        private const val PREFERENCES_NAME = "AndroidMicUserSettings"

        private const val IP_KEY = "DEFAULT_IP"
        private const val PORT_KEY = "DEFAULT_PORT"
        const val DEFAULT_IP = "192.168."
        const val DEFAULT_PORT = "55555"

        private const val MODE_KEY = "DEFAULT_MODE"
        private val DEFAULT_MODE = Modes.WIFI.ordinal

        private const val THEME_KEY = "DEFAULT_THEME"
        private val DEFAULT_THEME = Themes.SYSTEM.ordinal

        private const val DYNAMIC_COLOR_KEY = "DEFAULT_DYNAMIC_COLOR"
        private const val DEFAULT_DYNAMIC_COLOR = true

        private const val SAMPLE_RATE_KEY = "DEFAULT_SAMPLE_RATE"
        private val DEFAULT_SAMPLE_RATE = SampleRates.S16000.ordinal
    }

    fun setIpPort(ip: String, port: String) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putString(IP_KEY, ip)
        editor.putString(PORT_KEY, port)
        editor.apply()
    }

    fun getIpPort(): Pair<String, String> {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val ip = userSettings.getString(IP_KEY, DEFAULT_IP) ?: DEFAULT_IP
        val port = try {
            // backward compat
            userSettings.getString(PORT_KEY, DEFAULT_PORT) ?: DEFAULT_PORT
        } catch (e: Exception) {
            DEFAULT_PORT
        }
        return ip to port
    }

    fun setMode(mode: Modes) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(MODE_KEY, mode.ordinal)
        editor.apply()
    }

    fun getMode(): Modes {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        return userSettings.getInt(MODE_KEY, DEFAULT_MODE).let {
            Modes.values()[it]
        }
    }


    fun setTheme(theme: Themes) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(THEME_KEY, theme.ordinal)
        editor.apply()
    }

    fun getTheme(): Themes {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        return userSettings.getInt(THEME_KEY, DEFAULT_THEME).let {
            Themes.values()[it]
        }
    }


    fun setDynamicColor(dynamicColor: Boolean) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putBoolean(DYNAMIC_COLOR_KEY, dynamicColor)
        editor.apply()
    }

    fun getDynamicColor(): Boolean {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        return userSettings.getBoolean(DYNAMIC_COLOR_KEY, DEFAULT_DYNAMIC_COLOR)
    }

    fun setSampleRate(sampleRate: SampleRates) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(SAMPLE_RATE_KEY, sampleRate.ordinal)
        editor.apply()
    }

    fun getSampleRate(): SampleRates {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        return userSettings.getInt(SAMPLE_RATE_KEY, DEFAULT_SAMPLE_RATE).let {
            SampleRates.values()[it]
        }
    }
}