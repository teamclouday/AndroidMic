package io.github.teamclouday.androidMic

import android.content.Context
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import io.github.teamclouday.androidMic.ui.utils.UiHelper
import io.github.teamclouday.androidMic.utils.PreferencesManager

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
    val mode = enumPreference("mode", Mode.WIFI)

    val ip = stringPreference("ip", "192.168.")
    val port = stringPreference("port", "")


    val theme = enumPreference("theme", Themes.System)
    val dynamicColor = booleanPreference("dynamicColor", true)

    val sampleRate = enumPreference("sampleRate", SampleRates.S44100)
    val channelCount = enumPreference("channelCount", ChannelCount.Mono)
    val audioFormat = enumPreference("audioFormat", AudioFormat.I16)

}

enum class Mode {
    WIFI, UDP, USB, ADB
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

enum class AudioFormat(val value: Int, val description: String) {
    I8(android.media.AudioFormat.ENCODING_PCM_8BIT, "u8"),
    I16(android.media.AudioFormat.ENCODING_PCM_16BIT, "i16"),

    @RequiresApi(Build.VERSION_CODES.S)
    I24(android.media.AudioFormat.ENCODING_PCM_24BIT_PACKED, "i24"),

    @RequiresApi(Build.VERSION_CODES.S)
    I32(android.media.AudioFormat.ENCODING_PCM_32BIT, "i32"),
    F32(android.media.AudioFormat.ENCODING_PCM_FLOAT, "f32");

    override fun toString(): String = description
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

    fun getString(uiHelper: UiHelper): String {
        return when (this) {
            Mono -> uiHelper.getString(R.string.mono)
            Stereo -> uiHelper.getString(R.string.stereo)
        }
    }
}
