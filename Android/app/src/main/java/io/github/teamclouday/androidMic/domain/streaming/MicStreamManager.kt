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

    fun start(audioStream: Flow<AudioPacket>, tx: Messenger): Boolean {
        val connected = streamer.connect()
        if (connected) {
            streamer.start(audioStream, tx)
        }
        return connected
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