package com.example.androidMic.domain.streaming

import android.content.Context
import com.example.androidMic.domain.audio.AudioBuffer
import com.example.androidMic.ui.Modes

// StreamManager acts as a minimal RTSP server for audio data
// reference: https://www.medialan.de/usecase0001.html

// manage streaming data
class MicStreamManager(ctx: Context, val mode: Modes, ip: String?, port: Int?) {

    private var streamer: Streamer

    init {
        streamer = when (mode) {
            Modes.WIFI -> {
                WifiStreamer(ctx, ip!!, port!!)
            }

            Modes.BLUETOOTH -> {
                BluetoothStreamer(ctx)
            }

            Modes.USB -> {
                AdbStreamer(port!!)
            }

            Modes.UDP -> {
                UdpStreamer(ip!!, port!!)
            }
        }
    }


    companion object {
        const val STREAM_DELAY = 1L
    }

    fun start(): Boolean {
        return streamer.connect()
    }

    fun stop() {
        streamer.disconnect()
    }

    suspend fun stream(audioBuffer: AudioBuffer) {
        streamer.stream(audioBuffer)
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