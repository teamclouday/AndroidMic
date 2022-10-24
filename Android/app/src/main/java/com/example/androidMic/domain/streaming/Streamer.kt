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
        val DEVICE_CHECK = "AndroidMicCheck"
        val DEVICE_CHECK_EXPECT = "AndroidMicCheckAck"
        val BUFFER_SIZE = 1024
    }
}