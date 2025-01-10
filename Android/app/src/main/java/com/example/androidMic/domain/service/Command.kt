package com.example.androidMic.domain.service

import android.os.Bundle
import android.os.Message
import com.example.androidMic.AudioFormat
import com.example.androidMic.ChannelCount
import com.example.androidMic.Mode
import com.example.androidMic.SampleRates


private const val ID_MSG: String = "ID_MSG"
private const val ID_STATE: String = "ID_STATE"

private const val ID_MODE: String = "ID_MODE"

private const val ID_IP: String = "ID_IP"
private const val ID_PORT: String = "ID_PORT"

private const val ID_SAMPLE_RATE: String = "ID_SAMPLE_RATE"
private const val ID_CHANNEL_COUNT: String = "ID_CHANNEL_COUNT"
private const val ID_AUDIO_FORMAT: String = "ID_AUDIO_FORMAT"


/**
 * Commands UI -> Service
 */
enum class Command {
    StartStream,
    StopStream,
    GetStatus,
    // called when the ui is bind
    BindCheck,
}

fun Bundle.getOrdinal(key: String) : Int? {
    val v = this.getInt(key, Int.MIN_VALUE);

    return if (v == Int.MIN_VALUE) {
        null
    } else {
        v
    }
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
                port = msg.data.getInt(ID_PORT),
                sampleRate = msg.data.getOrdinal(ID_SAMPLE_RATE)?.let { SampleRates.entries[it] },
                channelCount = msg.data.getOrdinal(ID_CHANNEL_COUNT)?.let { ChannelCount.entries[it] },
                audioFormat = msg.data.getOrdinal(ID_AUDIO_FORMAT)?.let { AudioFormat.entries[it] },
            )
        }
    }

    fun toCommandMsg(): Message {

        val r = Bundle()

        this.mode?.let { r.putInt(ID_MODE, it.ordinal) }

        this.ip?.let { r.putString(ID_IP, it) }
        this.port?.let { r.putInt(ID_PORT, it) }

        this.sampleRate?.let { r.putInt(ID_SAMPLE_RATE, it.ordinal) }
        this.channelCount?.let { r.putInt(ID_CHANNEL_COUNT, it.ordinal) }
        this.audioFormat?.let { r.putInt(ID_AUDIO_FORMAT, it.ordinal) }

        val reply = Message.obtain()
        reply.data = r
        reply.what = this.command.ordinal

        return reply
    }


}


/**
 * Response Service -> UI
 */
enum class Response {
    Standard,
}

/**
 * Response Service -> UI
 */
enum class ServiceState {
    Connected,
    Disconnected,
}

data class ResponseData (
    val state: ServiceState? = null,
    val msg: String? = null,
    val kind: Response = Response.Standard,
) {



    companion object {
        fun fromMessage(msg: Message): ResponseData {
            return ResponseData(
                kind = Response.entries[msg.what],
                state = msg.data.getOrdinal(ID_STATE)?.let { ServiceState.entries[it] },
                msg = msg.data.getString(ID_MSG)
            )
        }
    }


    fun toResponseMsg(): Message {

        val r = Bundle()

        this.msg?.let { r.putString(ID_MSG, it) }
        this.state?.let { r.putInt(ID_STATE, it.ordinal) }

        val reply = Message.obtain()
        reply.data = r
        reply.what = kind.ordinal

        return reply
    }

}



