package com.example.androidMic.ui


import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger


object DefaultStates {
    const val IP = "192.168."
    const val PORT = "55555"
}

sealed interface States : java.io.Serializable {
    data class UiStates(
        val isStreamStarted: Boolean = false,
        val isAudioStarted: Boolean = false,
        val mode: Modes = Modes.WIFI,

        val switchAudioIsClickable: Boolean = true,
        val buttonConnectIsClickable: Boolean = true,

        val ip: String = DefaultStates.IP,
        val port: String = DefaultStates.PORT,

        val textLog: String = "",

        val dialogVisible: Dialogs = Dialogs.None,

        val theme: Themes = Themes.System,
        val dynamicColor: Boolean = true,
        val sampleRate: SampleRates = SampleRates.S16000,
        val channelCount: ChannelCount = ChannelCount.Mono,
        val audioFormat: AudioFormat = AudioFormat.I16,
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
    System,
    Dark,
    Light
}

enum class Dialogs {
    Modes,
    IpPort,
    Themes,
    SampleRates,
    ChannelCount,
    AudioFormat,
    None
}

// well, this can go crazy: https://github.com/audiojs/sample-rate
enum class SampleRates(val value: Int) {
    S8000(8000),
    S11025(11025),
    S16000(16000),
    S22050(22050),
    S44100(44100),
    S48000(48000),
    S88200(88200),
    S96600(96600),
    S176400(176400),
    S192000(192000),
    S352800(352800),
    S384000(384000),
}

enum class AudioFormat(val value: Int) {
    I16(1),
    I24(3),
    I32(4)
}

enum class ChannelCount(val value: Int) {
    Mono(1),
    Stereo(2),
}
