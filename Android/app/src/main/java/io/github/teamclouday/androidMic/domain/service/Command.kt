package io.github.teamclouday.androidMic.domain.service

import android.os.Bundle
import android.os.Message
import io.github.teamclouday.androidMic.AppPreferences
import io.github.teamclouday.androidMic.AudioFormat
import io.github.teamclouday.androidMic.ChannelCount
import io.github.teamclouday.androidMic.Dialogs
import io.github.teamclouday.androidMic.Mode
import io.github.teamclouday.androidMic.SampleRates
import io.github.teamclouday.androidMic.utils.Either
import io.github.teamclouday.androidMic.utils.checkIp
import io.github.teamclouday.androidMic.utils.checkPort

/**
 * Service -> UI
 */
private const val ID_MSG: String = "ID_MSG"
private const val ID_CONNECTION_STATE: String = "ID_CONNECTION_STATE"
private const val ID_MUTE_STATE: String = "ID_MUTE_STATE"

/**
 * UI -> Service
 */
private const val ID_MODE: String = "ID_MODE"

private const val ID_IP: String = "ID_IP"
private const val ID_PORT: String = "ID_PORT"

private const val ID_SAMPLE_RATE: String = "ID_SAMPLE_RATE"
private const val ID_CHANNEL_COUNT: String = "ID_CHANNEL_COUNT"
private const val ID_AUDIO_FORMAT: String = "ID_AUDIO_FORMAT"


/**
 * UI -> Service
 */
enum class Command {
    StartStream,
    StopStream,
    GetStatus,

    // called when the ui is bind
    BindCheck,

    Mute,
    Unmute,
}


data class CommandData(
    val command: Command,
    val mode: Mode? = null,
    var ip: String? = null,
    var port: Int? = null,
    val sampleRate: SampleRates? = null,
    val channelCount: ChannelCount? = null,
    val audioFormat: AudioFormat? = null,
) {

    companion object {
        fun fromMessage(msg: Message): CommandData {
            return CommandData(
                command = Command.entries[msg.what],
                mode = msg.data.getOrdinal(ID_MODE)?.let { Mode.entries[it] },
                ip = msg.data.getString(ID_IP),
                port = msg.data.getInt(ID_PORT).let { if (it == -1) null else it },
                sampleRate = msg.data.getOrdinal(ID_SAMPLE_RATE)?.let { SampleRates.entries[it] },
                channelCount = msg.data.getOrdinal(ID_CHANNEL_COUNT)
                    ?.let { ChannelCount.entries[it] },
                audioFormat = msg.data.getOrdinal(ID_AUDIO_FORMAT)?.let { AudioFormat.entries[it] },
            )
        }

        suspend fun fromPref(
            prefs: AppPreferences,
            command: Command
        ): Either<CommandData, Dialogs> {
            val ip = prefs.ip.get()
            val port = prefs.port.get()
            val mode = prefs.mode.get()

            val data = CommandData(
                command = command,
                sampleRate = prefs.sampleRate.get(),
                channelCount = prefs.channelCount.get(),
                audioFormat = prefs.audioFormat.get(),
                mode = mode
            )

            when (mode) {
                Mode.WIFI, Mode.UDP -> {
                    if (!checkIp(ip) || !checkPort(port)) {

                        return Either.Right(Dialogs.IpPort)
                    }
                    data.ip = ip
                    data.port = try {
                        port.toInt()
                    } catch (_: Exception) {
                        null
                    }
                }

                Mode.ADB -> {
                    if (!checkPort(port)) {
                        return Either.Right(Dialogs.Port)
                    }
                    data.port = try {
                        port.toInt()
                    } catch (_: Exception) {
                        null
                    }
                }

                else -> {}
            }

            return Either.Left(data)
        }
    }

    fun toCommandMsg(): Message {

        val data = Bundle()

        this.mode?.let { data.putInt(ID_MODE, it.ordinal) }

        this.ip?.let { data.putString(ID_IP, it) }
        data.putInt(ID_PORT, this.port ?: -1)

        this.sampleRate?.let { data.putInt(ID_SAMPLE_RATE, it.ordinal) }
        this.channelCount?.let { data.putInt(ID_CHANNEL_COUNT, it.ordinal) }
        this.audioFormat?.let { data.putInt(ID_AUDIO_FORMAT, it.ordinal) }

        val message = Message.obtain()
        message.data = data
        message.what = this.command.ordinal

        return message
    }


}


/**
 * Service -> UI
 */
enum class ResponseKind {
    Standard,
}

private enum class ConnectionState {
    Connected,
    Disconnected;
}

private enum class MuteState {
    Muted,
    Unmuted,
}

data class ResponseData(
    val msg: String? = null,
    val isConnected: Boolean? = null,
    val isMuted: Boolean? = null,
    val kind: ResponseKind = ResponseKind.Standard,
) {


    companion object {
        fun fromMessage(msg: Message): ResponseData {
            return ResponseData(
                kind = ResponseKind.entries[msg.what],
                isConnected = msg.data.getOrdinal(ID_CONNECTION_STATE)
                    ?.let { ConnectionState.entries[it] == ConnectionState.Connected },
                isMuted = msg.data.getOrdinal(ID_MUTE_STATE)
                    ?.let { MuteState.entries[it] == MuteState.Muted },
                msg = msg.data.getString(ID_MSG)
            )
        }
    }


    fun toResponseMsg(): Message {

        val r = Bundle()

        this.msg?.let { r.putString(ID_MSG, it) }
        this.isConnected?.let {
            r.putInt(
                ID_CONNECTION_STATE,
                (if (it) ConnectionState.Connected else ConnectionState.Disconnected).ordinal
            )
        }
        this.isMuted?.let {
            r.putInt(
                ID_MUTE_STATE,
                (if (it) MuteState.Muted else MuteState.Unmuted).ordinal
            )
        }

        val reply = Message.obtain()
        reply.data = r
        reply.what = kind.ordinal

        return reply
    }

}


fun Bundle.getOrdinal(key: String): Int? {
    val v = this.getInt(key, Int.MIN_VALUE)

    return if (v == Int.MIN_VALUE) {
        null
    } else {
        v
    }
}
