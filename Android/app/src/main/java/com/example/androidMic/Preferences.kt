package com.example.androidMic

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.example.androidMic.AudioFormat.F32
import com.example.androidMic.AudioFormat.I16
import com.example.androidMic.AudioFormat.I24
import com.example.androidMic.AudioFormat.I32
import com.example.androidMic.utils.PreferencesManager

object DefaultStates {
    const val IP = "192.168."
    const val PORT = "55555"
}

/**
 * Rules: key should be upper case snake case
 * Ex: SAMPLE_RATE
 * The key should match the name in the app state
 */
class AppPreferences(
    context: Context
) : PreferencesManager(context, "settings") {


    val mode = enumPreference("mode", Modes.WIFI)


    val ip = stringPreference("ip", "192.168.")
    val port = stringPreference("port", "55555")


    val theme = enumPreference("theme", Themes.System)
    val dynamicColor = booleanPreference("dynamicColor", true)

    val sampleRate = enumPreference("sampleRate", SampleRates.S16000)
    val channelCount = enumPreference("channelCount", ChannelCount.Mono)
    val audioFormat = enumPreference("audioFormat", AudioFormat.I16)

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
    IpPort,
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
    I32(4),
    F32(2);

    override fun toString(): String {
        return when (this) {
            I16 -> "i16"
            I24 -> "i24"
            I32 -> "i32"
            F32 -> "f32"
        }
    }
}



enum class ChannelCount(val value: Int) {
    Mono(1),
    Stereo(2);

    @Composable
    fun getString(): String {

        return when (this) {
            Mono -> stringResource(R.string.mono)
            Stereo -> stringResource(R.string.stereo)
        }
    }
}
