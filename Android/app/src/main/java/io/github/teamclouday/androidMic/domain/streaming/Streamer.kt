package io.github.teamclouday.androidMic.domain.streaming

import android.os.Messenger
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.flow.Flow

const val CHECK_1 = "AndroidMic1"
const val CHECK_2 = "AndroidMic2"

const val DEFAULT_PORT: Int = 55555
const val MAX_PORT: Int = 55570

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    fun start(audioStream: Flow<AudioPacket>, tx: Messenger)
    fun getInfo(): String
    fun isAlive(): Boolean
}