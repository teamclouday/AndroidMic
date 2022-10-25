package com.example.androidMic.domain.streaming

import com.example.androidMic.domain.audio.AudioBuffer

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    suspend fun stream(audioBuffer: AudioBuffer)
    fun getInfo(): String
    fun isAlive(): Boolean

    companion object {
        const val DEVICE_CHECK = "AndroidMicCheck"
        const val DEVICE_CHECK_EXPECT = "AndroidMicCheckAck"
        const val BUFFER_SIZE = 1024
    }
}