package io.github.teamclouday.AndroidMic.domain.streaming

import android.os.Messenger
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import kotlinx.coroutines.flow.Flow

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    fun start(audioStream: Flow<AudioPacket>, tx: Messenger)
    fun getInfo(): String
    fun isAlive(): Boolean
}