package io.github.teamclouday.androidMic.domain.streaming

import android.content.Context
import android.os.Messenger
import io.github.teamclouday.androidMic.Mode
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow

// manage streaming data
class MicStreamManager(
    ctx: Context,
    scope: CoroutineScope,
    val mode: Mode,
    ip: String?,
    port: Int?
) {

    private var streamer: Streamer = when (mode) {
        Mode.WIFI -> {
            TcpStreamer.wifi(ctx, scope, ip!!, port!!)
        }

        Mode.ADB -> {
            TcpStreamer.adb(scope, port!!)
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