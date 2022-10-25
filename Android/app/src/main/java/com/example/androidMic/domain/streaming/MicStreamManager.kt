package com.example.androidMic.domain.streaming

import android.content.Context
import com.example.androidMic.domain.audio.AudioBuffer
import com.example.androidMic.utils.DebugModes
import com.example.androidMic.utils.Modes.Companion.MODE_BLUETOOTH
import com.example.androidMic.utils.Modes.Companion.MODE_USB
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI

// StreamManager acts as a minimal RTSP server for audio data
// reference: https://www.medialan.de/usecase0001.html

// manage streaming data
class MicStreamManager(private val ctx: Context) {
    private val TAG: String = "MicStream"

    private var streamer: Streamer? = null

    private var mode: Int = -1

    companion object {
        const val STREAM_DELAY = 1L
    }

    fun initialize(mode: Int, ip: String?, port: Int?) {
        if (isConnected())
            throw IllegalArgumentException("Streaming already running")

        streamer = when (mode) {
            MODE_WIFI -> {
                WifiStreamer(ctx, ip, port)
            }
            MODE_BLUETOOTH -> {
                BluetoothStreamer(ctx)
            }
            MODE_USB -> {
                UsbAdbStreamer(ctx, 6000)
            }
            else -> throw IllegalArgumentException("Unknown mode")
        }
        this.mode = mode
    }

    fun start(): Boolean {
        return streamer?.connect() ?: false
    }

    fun stop() {
        streamer?.disconnect()
    }

    suspend fun stream(audioBuffer: AudioBuffer) {
        streamer?.stream(audioBuffer)
    }

    fun shutdown() {
        streamer?.shutdown()
        streamer = null
    }

    fun getInfo(): String {
        val debugModes = DebugModes()
        return "[Streaming Mode] ${debugModes.dic[mode]}\n${streamer?.getInfo()}"
    }

    fun isConnected(): Boolean {
        return streamer?.isAlive() == true
    }
}