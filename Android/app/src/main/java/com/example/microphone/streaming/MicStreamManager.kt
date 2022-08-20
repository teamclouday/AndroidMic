package com.example.microphone.streaming

import android.content.Context
import android.util.Log
import com.example.microphone.audio.AudioBuffer
import java.net.InetSocketAddress

// StreamManager acts as a minimal RTSP server for audio data
// reference: https://www.medialan.de/usecase0001.html

// manage streaming data
class MicStreamManager(private val ctx: Context) {
    private val TAG: String = "MicStream"

    enum class ConnectMode {
        NONE,
        BLUETOOTH,
        WIFI
    }

    private var mode = ConnectMode.NONE
    private var streamer: Streamer? = null

    companion object {
        const val STREAM_DELAY = 1L
        const val DEFAULT_IP = "192.168."
        const val DEFAULT_PORT = 55555
    }

    fun initialize() {
        if (isConnected())
            throw IllegalArgumentException("Streaming already running")
        mode = ConnectMode.NONE
        // try bluetooth first
        var err = ""
        try {
            streamer = BluetoothStreamer(ctx)
        } catch (e: IllegalArgumentException) {
            err += e.message
            err += " (AND) "
            streamer = null
        }
        if (streamer != null) {
            mode = ConnectMode.BLUETOOTH
            return
        }
        Log.d(TAG, "bluetooth server failed: $err")
        // next try WIFI
        try {
            streamer = WifiStreamer(ctx)
        } catch (e: IllegalArgumentException) {
            err += e.message
        }
        if (streamer != null) {
            mode = ConnectMode.WIFI
            return
        }
        // throw error
        throw IllegalArgumentException(err)
    }

    fun start(): Boolean {
        return streamer?.connect() ?: false
    }

    fun stop() {
        streamer?.disconnect()
    }

    fun needInitIP(): Boolean {
        return mode == ConnectMode.WIFI
    }

    suspend fun stream(audioBuffer: AudioBuffer) {
        streamer?.stream(audioBuffer)
    }

    fun shutdown() {
        mode = ConnectMode.NONE
        streamer?.shutdown()
        streamer = null
    }

    fun getInfo(): String {
        return "[Streaming Mode] ${mode.name}\n${streamer?.getInfo()}"
    }

    fun isConnected(): Boolean {
        return streamer?.isAlive() == true
    }

    fun setIPInfo(address: InetSocketAddress) {
        streamer?.updateAddress(address)
    }
}