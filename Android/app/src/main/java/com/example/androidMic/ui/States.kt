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

        val ip: String = "",
        val port: String = "",

        val textLog: String = "",

        val dialogVisible: Dialogs = Dialogs.None,

        val theme: Themes = Themes.SYSTEM,
        val dynamicColor: Boolean = true,
        val sampleRate: SampleRates = SampleRates.S16000,
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

enum class Dialogs {
    Modes,
    IpPort,
    Themes,
    SampleRates,
    None
}

enum class SampleRates(val value: Int) {
    S16000(16000),
    S48000(48000)
}