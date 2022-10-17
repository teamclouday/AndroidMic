package com.example.androidmic.ui.utils

import androidx.appcompat.app.AppCompatActivity
import com.example.androidmic.AndroidMicApp
import com.example.androidmic.R
import com.example.androidmic.utils.Modes
import com.example.androidmic.utils.Modes.Companion.MODE_WIFI
import java.net.InetSocketAddress

private const val PREFERENCES_NAME = "AndroidMicUserSettings"

private const val USER_SETTINGS_DEFAULT_IP_KEY = "DEFAULT_IP"
private const val USER_SETTINGS_DEFAULT_PORT_KEY = "DEFAULT_PORT"
private const val USER_SETTINGS_DEFAULT_MODE_KEY = "DEFAULT_MODE"


private const val DEFAULT_IP = "192.168."
private const val DEFAULT_PORT = 55555

private const val DEFAULT_MODE = MODE_WIFI

class Preferences(private val androidMicApp: AndroidMicApp) {

    fun setIpPort(pair: Pair<String, String>) {
        val ip = pair.first
        val port = pair.second.toInt()
        InetSocketAddress(ip, port)

        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE)

        val editor = userSettings.edit()
        editor.putString(USER_SETTINGS_DEFAULT_IP_KEY, ip)
        editor.putInt(USER_SETTINGS_DEFAULT_PORT_KEY, port)
        editor.apply()
    }

    fun getIpPort(withDefaultValue: Boolean): Pair<String, Int> {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE)

        var ip: String?
        val port: Int

        if(withDefaultValue) {
            ip = userSettings.getString(USER_SETTINGS_DEFAULT_IP_KEY, DEFAULT_IP)
            port = userSettings.getInt(USER_SETTINGS_DEFAULT_PORT_KEY, DEFAULT_PORT)
        }
        else {
            ip = userSettings.getString(USER_SETTINGS_DEFAULT_IP_KEY, "")
            port = userSettings.getInt(USER_SETTINGS_DEFAULT_PORT_KEY, 0)
        }

        // case if we want to send ip/port to service
        if(!withDefaultValue && (ip.isNullOrEmpty() || port == 0)) {
            throw IllegalArgumentException()
        }

        if(ip == null)
            ip = ""
        return ip to port
    }

    fun setMode(mode: Int) {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE)

        val editor = userSettings.edit()
        editor.putInt(USER_SETTINGS_DEFAULT_MODE_KEY, mode)
        editor.apply()
    }

    fun getMode(): Int {
        val userSettings = androidMicApp.getSharedPreferences(
            PREFERENCES_NAME,
            AppCompatActivity.MODE_PRIVATE)

        return userSettings.getInt(USER_SETTINGS_DEFAULT_MODE_KEY, DEFAULT_MODE)
    }


    fun getModeText(mode: Int): String {
        return when(mode) {
            MODE_WIFI -> androidMicApp.getString(R.string.mode_wifi)
            Modes.MODE_BLUETOOTH -> androidMicApp.getString(R.string.mode_bluetooth)
            Modes.MODE_USB -> androidMicApp.getString(R.string.mode_usb)
            else -> {"NONE"}}
    }
}