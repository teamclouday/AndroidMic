package io.github.teamclouday.AndroidMic.domain.streaming

import android.content.Context
import android.os.Messenger
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow

// manage streaming data
class MicStreamManager(
    ctx: Context,
    val scope: CoroutineScope,
    val mode: Mode,
    ip: String?,
    port: Int?
) {

    private var streamer: Streamer = when (mode) {
        Mode.WIFI -> {
            WifiStreamer(ctx, scope, ip!!, port!!)
        }

        Mode.ADB -> {
            AdbStreamer(scope)
        }

        Mode.USB -> {
            UsbStreamer(ctx, scope)
        }

        Mode.UDP -> {
            UdpStreamer(scope, ip!!, port!!)
        }
    }


    companion object {
        const val STREAM_DELAY = 1L
    }

    fun connect(): Boolean {
        return streamer.connect()
    }

    fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamer.start(audioStream, tx)
    }

    fun stop() {
        streamer.disconnect()
    }

    // should not call any methods after calling
    fun shutdown() {
        streamer.shutdown()
    }

    fun getInfo(): String {
        return "[Streaming Mode] ${mode.name}\n${streamer.getInfo()}"
    }

    fun isConnected(): Boolean {
        return streamer.isAlive()
    }
}