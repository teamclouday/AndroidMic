package com.example.androidmic.domain.streaming

import com.example.androidmic.domain.audio.AudioBuffer
import java.net.InetSocketAddress

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    suspend fun stream(audioBuffer: AudioBuffer)
    fun getInfo(): String
    fun isAlive(): Boolean

    companion object {
        val DEVICE_CHECK = "AndroidMicCheck"
        val DEVICE_CHECK_EXPECT = "AndroidMicCheckAck"
        val BUFFER_SIZE = 1024
    }
}