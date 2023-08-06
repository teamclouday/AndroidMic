package com.example.androidMic.utils


import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.Themes.Companion.SYSTEM_THEME
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

sealed interface States : java.io.Serializable {
    data class UiStates(
        val isStreamStarted: Boolean = false,
        val isAudioStarted: Boolean = false,
        val mode: Int = MODE_WIFI,

        val switchAudioIsClickable: Boolean = true,
        val buttonConnectIsClickable: Boolean = true,

        val IP: String = "",
        val PORT: String = "",

        val usbPort: String = "",

        val textLog: String = "",

        val dialogModesIsVisible: Boolean = false,
        val dialogIpPortIsVisible: Boolean = false,
        val dialogUsbPortIsVisible: Boolean = false,
        val dialogThemeIsVisible: Boolean = false,

        val theme: Int = SYSTEM_THEME,
        val dynamicColor: Boolean = true
    ) : States

    data class ServiceStates(
        var isStreamStarted: AtomicBoolean = AtomicBoolean(false),
        var streamShouldStop: AtomicBoolean = AtomicBoolean(false),
        var isAudioStarted: AtomicBoolean = AtomicBoolean(false),
        var audioShouldStop: AtomicBoolean = AtomicBoolean(false),
        var mode: AtomicInteger = AtomicInteger(MODE_WIFI)
    ) : States
}

class Modes {
    companion object {
        const val MODE_WIFI: Int = 1
        const val MODE_BLUETOOTH: Int = 2
        const val MODE_USB: Int = 3
        const val MODE_UDP: Int = 4
    }
}

class Themes {
    companion object {
        const val SYSTEM_THEME: Int = 0
        const val DARK_THEME: Int = 1
        const val LIGHT_THEME: Int = 2
    }
}