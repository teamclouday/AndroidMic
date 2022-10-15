package com.example.androidmic.utils


import com.example.androidmic.utils.Modes.Companion.MODE_WIFI
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

sealed interface States :java.io.Serializable {
    data class UiStates(
        val isStreamStarted: Boolean = false,
        val isAudioStarted: Boolean = false,
        val mode: Int = MODE_WIFI,

        val switchAudioIsClickable: Boolean = true,
        val buttonConnectIsClickable: Boolean = true,

        val IP: String = "",
        val PORT: String = "",
        val textMode: String = "",

        val textLog: String = "",

        val dialogModesIsVisible: Boolean = false,
        val dialogIpPortIsVisible: Boolean = false
    ): States

    data class ServiceStates(
        var isStreamStarted: AtomicBoolean = AtomicBoolean(false),
        var streamShouldStop: AtomicBoolean = AtomicBoolean(false),
        var isAudioStarted: AtomicBoolean = AtomicBoolean(false),
        var audioShouldStop: AtomicBoolean = AtomicBoolean(false),
        var mode: AtomicInteger = AtomicInteger(MODE_WIFI)
    ): States
}

class Modes {
    companion object {
        const val MODE_WIFI: Int = 1
        const val MODE_BLUETOOTH: Int = 2
        const val MODE_USB: Int = 3
    }
}