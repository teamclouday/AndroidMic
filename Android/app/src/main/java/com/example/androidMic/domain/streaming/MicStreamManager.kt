package com.example.androidMic.domain.streaming

import android.content.Context
import com.example.androidMic.Modes
import com.example.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow

// manage streaming data
class MicStreamManager(
    ctx: Context,
    val scope: CoroutineScope,
    val mode: Modes,
    ip: String?,
    port: Int?
) {

    private var streamer: Streamer = when (mode) {
        Modes.WIFI -> {
            WifiStreamer(ctx, scope, ip!!, port!!)
        }

        Modes.ADB -> {
            AdbStreamer(scope)
        }

        Modes.USB -> {
            UsbStreamer(ctx, scope)
        }

        Modes.UDP -> {
            UdpStreamer(scope, ip!!, port!!)
        }
    }


    companion object {
        const val STREAM_DELAY = 1L
    }

    fun start(audioStream: Flow<AudioPacket>): Boolean {
        val connected = streamer.connect()
        if (connected) {
            streamer.start(audioStream)
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