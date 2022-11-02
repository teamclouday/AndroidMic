package com.example.androidMic.ui.utils

import androidx.appcompat.app.AppCompatActivity
import com.example.androidMic.AndroidMicApp
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.Themes.Companion.SYSTEM_THEME
import java.net.InetSocketAddress


class Preferences(private val androidMicApp: AndroidMicApp) {

    companion object {
        private const val PREFERENCES_NAME = "AndroidMicUserSettings"

        private const val IP_KEY = "DEFAULT_IP"
        private const val PORT_KEY = "DEFAULT_PORT"
        const val DEFAULT_IP = "192.168."
        const val DEFAULT_PORT = 55555

        private const val USB_PORT_KEY = "DEFAULT_USB_PORT"
        const val DEFAULT_USB_PORT = 6000

        private const val MODE_KEY = "DEFAULT_MODE"
        private const val DEFAULT_MODE = MODE_WIFI

        private const val THEME_KEY = "DEFAULT_THEME"
        private const val DEFAULT_THEME = SYSTEM_THEME

        private const val DYNAMIC_COLOR_KEY = "DEFAULT_DYNAMIC_COLOR"
        private const val DEFAULT_DYNAMIC_COLOR = true
    }

    fun setWifiIpPort(pair: Pair<String, String>) {
        val ip = pair.first
        val port = pair.second.toInt()
        InetSocketAddress(ip, port)

        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putString(IP_KEY, ip)
        editor.putInt(PORT_KEY, port)
        editor.apply()
    }

    fun getWifiIpPort(withDefaultValue: Boolean): Pair<String, Int> {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        var ip: String?
        val port: Int

        if (withDefaultValue) {
            ip = userSettings.getString(IP_KEY, DEFAULT_IP)
            port = userSettings.getInt(PORT_KEY, DEFAULT_PORT)
        } else {
            ip = userSettings.getString(IP_KEY, "")
            port = userSettings.getInt(PORT_KEY, 0)
        }

        // case if we want to send ip/port to service
        if (!withDefaultValue && (ip.isNullOrEmpty() || port == 0)) {
            throw IllegalArgumentException()
        }

        if (ip == null)
            ip = ""
        return ip to port
    }


    fun setUsbPort(_port: String) {
        val port = _port.toInt()
        InetSocketAddress("127.0.0.1", port)
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(USB_PORT_KEY, port)
        editor.apply()
    }

    fun getUsbPort(): Int {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        return userSettings.getInt(USB_PORT_KEY, DEFAULT_USB_PORT)
    }


    fun setMode(mode: Int) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(MODE_KEY, mode)
        editor.apply()
    }

    fun getMode(): Int {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        return userSettings.getInt(MODE_KEY, DEFAULT_MODE)
    }


    fun setTheme(theme: Int) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )

        val editor = userSettings.edit()
        editor.putInt(THEME_KEY, theme)
        editor.apply()
    }

    fun getTheme(): Int {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE
        )
        return userSettings.getInt(THEME_KEY, DEFAULT_THEME)
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
}