package com.example.androidMic.ui


import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

sealed interface States : java.io.Serializable {
    data class UiStates(
        val isStreamStarted: Boolean = false,
        val isAudioStarted: Boolean = false,
        val mode: Modes = Modes.WIFI,

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

        val theme: Themes = Themes.SYSTEM,
        val dynamicColor: Boolean = true
    ) : States

    data class ServiceStates(
        var isStreamStarted: AtomicBoolean = AtomicBoolean(false),
        var streamShouldStop: AtomicBoolean = AtomicBoolean(false),
        var isAudioStarted: AtomicBoolean = AtomicBoolean(false),
        var audioShouldStop: AtomicBoolean = AtomicBoolean(false),
        var mode: AtomicInteger = AtomicInteger(Modes.WIFI.ordinal)
    ) : States
}

enum class Modes {
    WIFI, BLUETOOTH, USB, UDP
}

enum class Themes {
    SYSTEM, DARK, LIGHT
}